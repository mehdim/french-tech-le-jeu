//! La simulation de référence, déterministe.
//!
//! Un `tick()` = un jour de jeu. Ordre des opérations :
//!  1. les salariés produisent du travail   6. les revenus s'accumulent
//!  2. chantiers / paperasse / pitchs       7. les dépenses s'accumulent
//!  3. le marketing génère des leads        8. moral, fatigue, départs
//!  4. les commerciaux convertissent        9. événements (différés, majeurs, aléatoires)
//!  5. des clients résilient               10. les objectifs sont évalués

use crate::content::{Content, ProgramKind, RoundKind, Trait};
use crate::customers::{FunnelInputs, MarketState, daily_funnel};
use crate::economy::{Finance, Metrics, MonthlyReport, RunRate, ValuationInputs, ltv, runway, safe_div, valuation};
use crate::employees::{Activity, Assignment, Candidate, Employee, RecruitmentSearch, Skills, default_assignment, roll_employee};
use crate::events::{DelayedEffect, EventRecord, EventState, PendingEvent};
use crate::funding::{AppStatus, Application, Disbursement, Fundraising, FundingState, Loan, MoneyKind, Offer, RaiseStatus, RoundRecord};
use crate::messages::{Cue, LogEntry, Msg, Tone};
use crate::missions::{CampaignState, LevelResult, LevelSnapshot, Phase};
use crate::office::{OfficeEffects, OfficeState};
use crate::product::ProductState;
use crate::research::{ActiveTask, Approach, task_availability};
use crate::types::{DAYS_PER_MONTH, ModKind, Modifier, Role, Segment, Seniority};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Incrémenté à chaque changement incompatible du format de sauvegarde.
pub const SAVE_SCHEMA: u32 = 1;
const LOG_CAP: usize = 250;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    pub hired: u32,
    pub fired: u32,
    pub resigned: u32,
    pub shipped: u32,
    pub incidents: u32,
    pub paperwork: f64,
    pub events: u32,
    pub max_mrr: f64,
    pub customers_won: u32,
    pub customers_lost: u32,
    /// Masse salariale R&D depuis la dernière demande de crédit d'impôt recherche.
    pub rd_spend: f64,
    pub coffees: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    pub schema: u32,
    pub seed: u64,
    pub rng: ChaCha8Rng,
    pub day: u32,
    pub company: String,
    pub reputation: f64,
    pub flags: BTreeSet<String>,
    pub employees: Vec<Employee>,
    pub candidates: Vec<Candidate>,
    pub searches: Vec<RecruitmentSearch>,
    pub tasks: Vec<ActiveTask>,
    pub product: ProductState,
    pub market: MarketState,
    pub finance: Finance,
    pub funding: FundingState,
    pub office: OfficeState,
    pub events: EventState,
    pub campaign: CampaignState,
    pub modifiers: Vec<Modifier>,
    pub crunch: bool,
    pub founder_mult: f64,
    /// Pression concurrentielle ajoutée par les événements ; décroît chaque mois.
    pub event_pressure: f64,
    pub next_id: u32,
    pub log: Vec<LogEntry>,
    pub incident_day: Option<u32>,
    pub stats: Stats,
    /// Compteurs hebdomadaires pour le fil d'actualité.
    pub week_new: u32,
    pub week_churn: u32,
    /// Production de l'équipe le dernier jour travaillé (pour les explications de l'interface).
    #[serde(default)]
    pub last_work: WorkSummary,
    /// Mentors d'A la French (absent des anciennes sauvegardes : valeur par défaut).
    #[serde(default)]
    pub mentors: crate::mentors::MentorState,
    /// Compteurs d'histoire (recrues à San Francisco, accords de Davos…).
    #[serde(default)]
    pub counters: BTreeMap<String, u32>,
    #[serde(skip)]
    pub metrics: Metrics,
    #[serde(skip)]
    pub cues: Vec<Cue>,
}

/// Commandes du joueur : le seul moyen pour la couche de présentation de modifier la partie.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Command {
    Assign { employee: u32, assignment: Assignment },
    StartTask { task: String, approach: Approach },
    CancelTask { instance: u32 },
    StartRecruitment { channel: String, role: Role },
    Hire { candidate: u32 },
    RejectCandidate { candidate: u32 },
    Fire { employee: u32 },
    SetPrice { segment: Segment, price: f64 },
    SetMarketingBudget(f64),
    SetCrunch(bool),
    StartApplication { program: String },
    StartFundraising { round: RoundKind },
    AcceptOffer { index: usize },
    DeclineOffers,
    TakeFriendsFamily,
    PlaceFurniture { item: String, x: i32, y: i32, rotated: bool },
    PlaceStored { index: usize, x: i32, y: i32, rotated: bool },
    MoveFurniture { uid: u32, x: i32, y: i32, rotated: bool },
    SellFurniture { uid: u32 },
    MoveOffice { tier: u32 },
    OpenMarket { market: String },
    ResolveEvent { choice: usize },
    DismissIntro,
    AdvanceLevel,
    ChooseMentor { mentor: String },
    ConsultMentor { mentor: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CmdError {
    Locked,
    NotFound,
    NotEnoughCash,
    Invalid(&'static str),
    Blocked(String),
}

impl CmdError {
    pub fn label(&self) -> String {
        match self {
            CmdError::Locked => "Pas encore débloqué".into(),
            CmdError::NotFound => "Introuvable".into(),
            CmdError::NotEnoughCash => "Trésorerie insuffisante".into(),
            CmdError::Invalid(s) => (*s).to_string(),
            CmdError::Blocked(s) => s.clone(),
        }
    }
}

/// Production du dernier jour travaillé, gardée pour l'interface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WorkSummary {
    pub sales: f64,
    pub marketing: f64,
    pub ops: f64,
    pub admin: f64,
    pub pitch: f64,
    pub growth_hackers: u32,
}

/// Travail quotidien produit par l'équipe.
#[derive(Clone, Debug, Default)]
struct Work {
    task_points: BTreeMap<u32, (f64, f64)>,
    sales: f64,
    marketing: f64,
    ops: f64,
    admin: f64,
    pitch: f64,
    task_workers: u32,
    hr: u32,
    cfo: bool,
    rust: bool,
    growth_hackers: u32,
}

impl GameState {
    // ------------------------------------------------------------------
    // construction
    // ------------------------------------------------------------------

    pub fn new(content: &Content, seed: u64) -> Self {
        let b = &content.balancing;
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let founder_arch = content.archetype("founder").expect("le contenu doit définir le fondateur");
        let mut founder = roll_employee(content, founder_arch, None, 1, 0, &mut rng);
        founder.name = "Vous".to_string();
        founder.salary = 0.0;
        founder.morale = 80.0;
        founder.fatigue = 5.0;
        founder.loyalty = 1.0;
        founder.productivity = 1.0;
        founder.skills = Skills { eng: 70.0, research: 50.0, sales: 40.0, marketing: 32.0, ops: 50.0 };
        founder.seniority = Seniority::Senior;
        founder.look = 0;
        let mut office = OfficeState::new(0);
        office.move_to(content, 0);
        let mut s = Self {
            schema: SAVE_SCHEMA,
            seed,
            rng,
            day: 0,
            company: "VERIDIX".to_string(),
            reputation: b.start_reputation,
            flags: BTreeSet::new(),
            employees: vec![founder],
            candidates: Vec::new(),
            searches: Vec::new(),
            tasks: Vec::new(),
            product: ProductState::new(b.start_accuracy, b.start_fpr, b.start_tech_debt, b.inference_cost_per_k),
            market: MarketState::new(content),
            finance: Finance::new(b.start_cash),
            funding: FundingState::new(),
            office,
            events: EventState::default(),
            campaign: CampaignState::new(),
            modifiers: Vec::new(),
            crunch: false,
            founder_mult: 1.0,
            event_pressure: 0.0,
            next_id: 2,
            log: Vec::new(),
            incident_day: None,
            stats: Stats::default(),
            week_new: 0,
            week_churn: 0,
            last_work: WorkSummary::default(),
            mentors: Default::default(),
            counters: BTreeMap::new(),
            metrics: Metrics::default(),
            cues: Vec::new(),
        };
        s.start_level(content, 1);
        s
    }

    /// Call after deserializing a save: rebuilds derived data.
    pub fn after_load(&mut self, content: &Content) {
        self.recompute_metrics(content);
    }

    fn new_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn log(&mut self, tone: Tone, msg: Msg) {
        self.log.push(LogEntry { day: self.day, tone, msg });
        if self.log.len() > LOG_CAP {
            let excess = self.log.len() - LOG_CAP;
            self.log.drain(0..excess);
        }
    }

    pub fn mod_mul(&self, kind: ModKind) -> f64 {
        self.modifiers.iter().filter(|m| m.kind == kind).map(|m| m.value).product()
    }

    pub fn mod_add(&self, kind: ModKind) -> f64 {
        self.modifiers.iter().filter(|m| m.kind == kind).map(|m| m.value).sum()
    }

    pub fn employee(&self, id: u32) -> Option<&Employee> {
        self.employees.iter().find(|e| e.id == id)
    }

    pub fn is_unlocked(&self, key: &str) -> bool {
        self.campaign.is_unlocked(key)
    }

    pub fn is_running(&self) -> bool {
        self.campaign.phase == Phase::Playing
            && self.events.pending.is_none()
            && !self.campaign.intro_pending
    }

    // ------------------------------------------------------------------
    // niveaux
    // ------------------------------------------------------------------

    pub fn start_level(&mut self, content: &Content, level: u32) {
        let Some(def) = content.level(level) else { return };
        self.campaign.level = level;
        self.campaign.level_start_day = self.day;
        self.campaign.quests_done = vec![false; def.quests.len()];
        self.campaign.boss_fired = false;
        self.campaign.shipped_in_level = 0;
        self.campaign.hired_in_level = 0;
        self.campaign.intro_pending = true;
        self.campaign.phase = Phase::Playing;
        for u in &def.unlocks {
            self.campaign.unlocks.insert(u.clone());
        }
        if let Some(min) = def.min_office
            && self.office.tier < min {
                self.office.move_to(content, min);
                let name = content.office(min).map(|o| o.name.clone()).unwrap_or_default();
                self.log(Tone::Story, Msg::OfficeMoved { name });
            }
        let effects = def.start_effects.clone();
        self.apply_effects(content, &effects, &def.title);
        self.recompute_metrics(content);
        self.campaign.snapshot = LevelSnapshot {
            paying_customers: self.paying_customers(),
            inference_cost_per_k: self.metrics.inference_cost_per_k,
            mrr: self.metrics.mrr,
            employees: self.employees.len() as u32,
            energy_per_k: self.product.energy_per_k,
        };
    }

    fn evaluate_missions(&mut self, content: &Content) {
        if self.campaign.phase != Phase::Playing {
            return;
        }
        let Some(def) = content.level(self.campaign.level) else { return };
        let mut all = true;
        let mut newly = Vec::new();
        for (i, q) in def.quests.iter().enumerate() {
            let ok = self.check(&q.condition);
            match q.kind {
                crate::content::QuestKind::Reach => {
                    if ok && !self.campaign.quests_done[i] {
                        self.campaign.quests_done[i] = true;
                        newly.push(q.label.clone());
                    }
                }
                crate::content::QuestKind::Maintain => self.campaign.quests_done[i] = ok,
            }
            all &= self.campaign.quests_done[i];
        }
        for label in newly {
            self.log(Tone::Good, Msg::QuestDone { label });
        }
        if all {
            self.campaign.results.push(LevelResult {
                level: self.campaign.level,
                days: self.day_in_level(),
                cash_end: self.finance.cash,
                mrr_end: self.metrics.mrr,
            });
            let last = content.levels.iter().map(|l| l.id).max().unwrap_or(12);
            if self.campaign.level >= last {
                self.campaign.ending = self
                    .flags
                    .iter()
                    .find_map(|f| f.strip_prefix("ending_").map(str::to_string));
                self.campaign.phase = Phase::Victory;
                self.cues.push(Cue::Victory);
            } else {
                self.campaign.phase = Phase::LevelComplete;
                self.cues.push(Cue::LevelComplete);
            }
            let level = self.campaign.level;
            self.log(Tone::Story, Msg::LevelComplete { level });
        }
    }

    // ------------------------------------------------------------------
    // le tick quotidien
    // ------------------------------------------------------------------

    /// Avance d'un jour. Renvoie false quand la simulation attend le
    /// joueur (choix d'événement, changement de niveau, intro, fin de partie).
    pub fn tick(&mut self, content: &Content) -> bool {
        if !self.is_running() {
            return false;
        }
        self.day += 1;
        let weekday = (self.day - 1) % 7;
        let working = weekday < 5 || self.crunch;
        let fx = self.office.effects(content);

        let work = self.compute_work(content, &fx, working);
        self.advance_tasks(content, &work);
        self.advance_paperwork(content, &work);
        self.advance_fundraising(content, &work);
        self.run_funnel(content, &work, &fx);
        self.accrue_money(content);
        self.update_people(content, &fx, &work, working);
        self.update_product(content, &work, &fx);
        self.apply_staffing(content, working);
        self.update_recruitment(content);
        self.decay_modifiers();
        self.update_events(content);
        if self.day.is_multiple_of(DAYS_PER_MONTH) {
            self.close_month(content);
        }
        if self.day.is_multiple_of(7) {
            self.finance.history.push((self.day, self.finance.cash, self.metrics.mrr));
            if self.week_new + self.week_churn > 0 {
                let (new, churned) = (self.week_new, self.week_churn);
                self.log(Tone::Info, Msg::CustomersWeek { new, churned });
            }
            self.week_new = 0;
            self.week_churn = 0;
        }
        self.recompute_metrics(content);
        self.check_bankruptcy(content);
        self.evaluate_missions(content);
        true
    }

    fn compute_work(&mut self, content: &Content, fx: &OfficeEffects, working: bool) -> Work {
        let b = &content.balancing;
        let mut w = Work::default();
        let day = self.day;
        let engineers = self.employees.iter().filter(|e| matches!(e.assignment, Assignment::Task(_))).count() as u32;
        w.task_workers = engineers;
        let pms = self
            .employees
            .iter()
            .filter(|e| e.role == Role::ProductManager && e.assignment == Assignment::Management && e.is_available(day))
            .count() as u32;
        w.hr = self
            .employees
            .iter()
            .filter(|e| e.role == Role::Hr && e.assignment == Assignment::Management && e.is_available(day))
            .count() as u32;
        w.cfo = self
            .employees
            .iter()
            .any(|e| e.role == Role::Cfo && e.assignment == Assignment::Management && e.is_available(day));
        w.rust = self.employees.iter().any(|e| e.trait_ == Trait::RustEvangelist);
        w.growth_hackers = self.employees.iter().filter(|e| e.trait_ == Trait::GrowthHacker).count() as u32;
        if !working {
            return w;
        }
        let useful_pms = pms.min(engineers.div_ceil(5));
        let pm_bonus = 1.0 + 0.07 * useful_pms as f64 - 0.04 * (pms - useful_pms) as f64;
        let meeting_heavy = engineers > 0 && pms as f64 / engineers as f64 > 0.25;
        let prod_mul = self.mod_mul(ModKind::ProductivityMul) * (1.0 + fx.productivity);
        let research_mul = self.mod_mul(ModKind::ResearchMul) * (1.0 + fx.research_bonus);
        let sales_mul = self.mod_mul(ModKind::SalesMul) * (1.0 + fx.sales_bonus);
        let seats = fx.seats.max(1) as f64;
        let headcount = self.employees.len() as f64;
        let seat_factor = if headcount > seats { (seats / headcount).sqrt() } else { 1.0 };
        let debt_drag = (1.0 - self.product.tech_debt * b.tech_debt_drag).max(0.3);
        let equipped = fx.workstations as f64 >= engineers as f64 * 0.5;
        let k = b.output_per_skill;

        for e in &self.employees {
            if !e.is_available(day) {
                continue;
            }
            let mut eff = e.productivity
                * e.morale_factor()
                * e.fatigue_factor()
                * e.seniority.output_mult()
                * prod_mul
                * seat_factor;
            if day < e.hired_day + b.onboarding_days && e.role != Role::Founder {
                eff *= 0.5;
            }
            if e.role == Role::Founder {
                eff *= self.founder_mult;
            }
            let (mut eng_m, mut res_m, mut mkt_m) = (1.0, 1.0, 1.0);
            match e.trait_ {
                Trait::Polytechnicien => {
                    res_m = 1.4;
                    eng_m = 1.1;
                }
                Trait::Dev42 => eng_m = if meeting_heavy { 0.8 } else { 1.3 },
                Trait::RustEvangelist => eng_m = 1.1,
                Trait::ExGafam => eng_m = if equipped { 1.5 } else { 1.1 },
                Trait::GrowthHacker => mkt_m = 1.7,
                Trait::PhdResearcher => res_m = 1.5,
                _ => {}
            }
            match e.assignment {
                Assignment::Unassigned => {}
                Assignment::Task(inst) => {
                    let eng = e.skills.eng * k * eff * eng_m * debt_drag * pm_bonus * (1.0 + fx.eng_bonus);
                    let res = e.skills.research * k * eff * res_m * research_mul * pm_bonus;
                    let slot = w.task_points.entry(inst).or_insert((0.0, 0.0));
                    slot.0 += eng;
                    slot.1 += res;
                }
                Assignment::Sales => w.sales += e.skills.sales * k * eff * sales_mul,
                Assignment::Marketing => w.marketing += e.skills.marketing * k * eff * mkt_m,
                Assignment::Ops => w.ops += e.skills.ops * k * eff,
                Assignment::Admin => {
                    let m = if e.role == Role::Cfo { 1.8 } else { 1.0 };
                    w.admin += (4.0 + e.skills.ops / 15.0) * eff * m;
                }
                Assignment::Fundraising => {
                    let m = if e.role == Role::Cfo { 1.3 } else { 1.0 };
                    w.pitch += (3.0 + e.skills.sales / 12.0) * eff * m;
                }
                Assignment::Management => {
                    if e.role == Role::Cfo {
                        w.admin += 2.5 * eff;
                    }
                }
            }
        }
        w
    }

    fn advance_tasks(&mut self, content: &Content, work: &Work) {
        let mut completed = Vec::new();
        for t in &mut self.tasks {
            if let Some(&(eng, res)) = work.task_points.get(&t.instance) {
                t.eng_done = (t.eng_done + eng).min(t.eng_total);
                t.research_done = (t.research_done + res).min(t.research_total);
            }
            if t.is_complete() {
                completed.push(t.instance);
            }
        }
        for inst in completed {
            let Some(pos) = self.tasks.iter().position(|t| t.instance == inst) else { continue };
            let task = self.tasks.remove(pos);
            let Some(def) = content.task(&task.task_id) else { continue };
            let effects = def.effects.clone();
            self.apply_effects(content, &effects, &def.name);
            let mut debt = def.debt * task.approach.debt_mult();
            if self.employees.iter().any(|e| e.trait_ == Trait::RustEvangelist) {
                debt *= 0.85;
            }
            self.product.tech_debt = (self.product.tech_debt + debt).clamp(0.0, 100.0);
            let risk = def.risk * task.approach.risk_mult();
            if risk > 0.0 && self.rng.random_bool(risk.clamp(0.0, 1.0)) {
                let extra = 4.0 + def.debt * 0.5;
                self.product.tech_debt = (self.product.tech_debt + extra).min(100.0);
                self.log(Tone::Bad, Msg::TaskComplication { task: def.name.clone(), debt: extra });
            }
            self.product.shipped.insert(def.id.clone());
            if def.meaningful {
                self.campaign.shipped_in_level += 1;
            }
            self.stats.shipped += 1;
            self.log(Tone::Good, Msg::TaskShipped { task: def.name.clone() });
            self.cues.push(Cue::Shipped(def.id.clone()));
            for e in &mut self.employees {
                if e.assignment == Assignment::Task(inst) {
                    e.assignment = Assignment::Unassigned;
                }
            }
        }
    }

    fn advance_paperwork(&mut self, content: &Content, work: &Work) {
        let mut admin = work.admin;
        self.stats.paperwork += admin;
        let day = self.day;
        let mut submitted = Vec::new();
        for a in self.funding.applications.iter_mut() {
            if a.status != AppStatus::Drafting || admin <= 0.0 {
                continue;
            }
            let need = (a.admin_required - a.admin_done).max(0.0);
            let used = need.min(admin);
            a.admin_done += used;
            admin -= used;
            if a.admin_done >= a.admin_required - 1e-6 {
                a.status = AppStatus::Submitted;
                a.submitted_day = Some(day);
                let wait = content.program(&a.program).map(|p| p.wait_days).unwrap_or(30);
                a.decision_day = Some(day + wait);
                submitted.push(a.program.clone());
            }
        }
        for p in submitted {
            self.flags.insert(format!("submitted_{p}"));
            let name = content.program(&p).map(|d| d.name.clone()).unwrap_or(p);
            self.log(Tone::Info, Msg::ApplicationSubmitted { program: name });
        }
        // Décisions.
        let decide: Vec<usize> = self
            .funding
            .applications
            .iter()
            .enumerate()
            .filter(|(_, a)| a.status == AppStatus::Submitted && a.decision_day.is_some_and(|d| d <= day))
            .map(|(i, _)| i)
            .collect();
        for i in decide {
            let program_id = self.funding.applications[i].program.clone();
            let Some(def) = content.program(&program_id) else { continue };
            let innovation = self.metrics.innovation_score;
            // Les comités jugent l'innovation et la crédibilité ; l'accompagnement des créateurs,
            // les statuts et les crédits d'impôt sont surtout une affaire de paperasse.
            let merit = match def.kind {
                ProgramKind::Grant | ProgramKind::Advance | ProgramKind::Loan => {
                    (0.8 + innovation / 250.0) * (0.9 + self.reputation / 250.0)
                }
                _ => 1.0,
            };
            let p = (def.base_success * merit * if work.cfo { 1.15 } else { 1.0 }).clamp(0.05, 0.98);
            let approved = self.rng.random_bool(p);
            let amount = self.funding.applications[i].amount;
            if approved {
                self.funding.applications[i].status = AppStatus::Approved;
                self.approve_program(content, &program_id, amount);
            } else {
                self.funding.applications[i].status = AppStatus::Rejected;
                if !def.reject_text.is_empty() {
                    self.log(Tone::Story, Msg::Text(def.reject_text.clone()));
                }
                self.log(Tone::Bad, Msg::ApplicationRejected { program: def.name.clone() });
            }
        }
    }

    fn approve_program(&mut self, content: &Content, program_id: &str, amount: f64) {
        let Some(def) = content.program(program_id) else { return };
        let day = self.day;
        let level = self.campaign.level;
        match def.kind {
            ProgramKind::Grant | ProgramKind::TaxCredit => {
                self.funding.secure(level, amount);
            }
            ProgramKind::Loan => {
                self.funding.secure(level, amount);
                self.funding.loans.push(Loan {
                    label: def.name.clone(),
                    monthly: amount * 1.03 / def.repay_months.max(1) as f64,
                    grace_left: def.grace_months,
                    months_left: def.repay_months,
                });
            }
            ProgramKind::Advance => {
                self.funding.secure(level, amount);
                self.funding.loans.push(Loan {
                    label: def.name.clone(),
                    monthly: amount * 0.5 / def.repay_months.max(1) as f64,
                    grace_left: def.grace_months,
                    months_left: def.repay_months,
                });
            }
            ProgramKind::Status => self.funding.jei = true,
            ProgramKind::FounderSupport => {}
        }
        for &(delay, frac) in &def.tranches {
            if amount * frac > 0.0 {
                self.funding.disbursements.push(Disbursement {
                    due_day: day + delay,
                    amount: amount * frac,
                    label: def.name.clone(),
                    kind: if def.kind == ProgramKind::Loan { MoneyKind::Loan } else { MoneyKind::Grant },
                });
            }
        }
        self.funding.approved.push(def.id.clone());
        let effects = def.on_approve.clone();
        self.apply_effects(content, &effects, &def.name);
        if !def.approve_text.is_empty() {
            self.log(Tone::Story, Msg::Text(def.approve_text.clone()));
        }
        self.log(Tone::Money, Msg::ApplicationApproved { program: def.name.clone(), amount });
    }

    fn advance_fundraising(&mut self, content: &Content, work: &Work) {
        let day = self.day;
        let Some(fr) = self.funding.fundraising.as_mut() else { return };
        match fr.status {
            RaiseStatus::Pitching => {
                fr.pitch_done += work.pitch;
                if fr.pitch_done >= fr.pitch_required {
                    let round = fr.round;
                    self.generate_offers(content, round);
                }
            }
            RaiseStatus::Offers => {
                if fr.offers.iter().all(|o| o.expires_day <= day) {
                    self.funding.fundraising = None;
                    self.log(Tone::Bad, Msg::OffersExpired);
                }
            }
        }
    }

    fn generate_offers(&mut self, content: &Content, round: RoundKind) {
        let Some(rdef) = content.round(round) else { return };
        let base = self.metrics.valuation;
        let mut offers = Vec::new();
        for inv in &content.investors {
            if !inv.rounds.contains(&round) || !inv.requires.iter().all(|c| self.check(c)) {
                continue;
            }
            let pre = base * inv.valuation_mult * self.rng.random_range(0.9..1.12);
            let d = self.rng.random_range(rdef.dilution.0..=rdef.dilution.1) / 100.0;
            let amount = (pre * d / (1.0 - d)).clamp(inv.ticket.0, inv.ticket.1);
            let amount = (amount / 10_000.0).round().max(1.0) * 10_000.0;
            offers.push(Offer {
                investor: inv.id.clone(),
                amount,
                pre_money: pre,
                dilution: 100.0 * amount / (pre + amount),
                expires_day: self.day + 21,
            });
        }
        if offers.is_empty() {
            self.funding.fundraising = None;
            self.reputation = (self.reputation - 1.0).max(0.0);
            self.log(Tone::Bad, Msg::NoOffers);
        } else {
            let n = offers.len() as u32;
            if let Some(fr) = self.funding.fundraising.as_mut() {
                fr.status = RaiseStatus::Offers;
                fr.offers = offers;
            }
            self.log(Tone::Money, Msg::OffersReceived { count: n });
            self.cues.push(Cue::EventPopup);
        }
    }

    fn market_mult(&self, content: &Content) -> f64 {
        self.market.markets.iter().filter_map(|m| content.market(m)).map(|m| m.lead_mult).sum::<f64>().max(1.0)
    }

    pub fn competition(&self, content: &Content) -> f64 {
        let b = &content.balancing;
        let base = b.competition_base + (self.campaign.level.saturating_sub(1)) as f64 * b.competition_per_level;
        let differentiation = (self.product.effective_accuracy() - 80.0).max(0.0) * 0.006 + self.reputation * 0.0008;
        (base + self.event_pressure + self.mod_add(ModKind::CompetitionAdd) - differentiation).clamp(0.0, 0.85)
    }

    pub fn support_deficit(&self, ops: f64, sales: f64) -> f64 {
        let customers = self.market.total_customers() as f64;
        let capacity = 500.0 + ops * 120.0 + sales * 20.0;
        (customers / capacity - 1.0).clamp(0.0, 1.5)
    }

    /// Entrées du tunnel à partir de l'état courant (production de l'équipe du dernier
    /// jour travaillé). Utilisé par le tunnel quotidien et par les détails de l'interface.
    pub fn funnel_inputs(&self, content: &Content) -> FunnelInputs {
        let b = &content.balancing;
        let mut gate = [0.0; 3];
        let mut unlocked = [false; 3];
        for s in Segment::ALL {
            let def = content.segment(s);
            unlocked[s.index()] = self.segment_unlocked(content, s);
            let n = def.required_features.len();
            gate[s.index()] = if n == 0 {
                1.0
            } else {
                def.required_features.iter().filter(|f| self.product.has(f)).count() as f64 / n as f64
            };
        }
        // Tant qu'aucun MVP n'existe, il n'y a rien à vendre, pas même une bêta.
        if !self.product.has("mvp_text") {
            unlocked = [false; 3];
        }
        let w = self.last_work;
        FunnelInputs {
            quality: self.product.quality(),
            fpr: self.product.effective_fpr(),
            reputation: self.reputation,
            launched: self.product.launched,
            sales_power: w.sales,
            marketing_power: w.marketing,
            lead_mul: self.mod_mul(ModKind::LeadMul),
            conv_mul: self.mod_mul(ModKind::ConversionMul) * 0.93f64.powi(w.growth_hackers as i32),
            churn_add: self.mod_add(ModKind::ChurnAdd),
            competition: self.competition(content),
            unlocked,
            feature_gate: gate,
            retention: self.product.retention,
            conversion_bonus: self.product.conversion,
            market_mult: self.market_mult(content),
            incident_recent: (self.product.incident_penalty / 25.0).clamp(0.0, 2.0),
            support_deficit: self.support_deficit(w.ops, w.sales),
            organic_leads: b.organic_leads,
            self_serve_rate: b.self_serve_rate,
            lead_decay: b.lead_decay,
            churn_quality_slope: b.churn_quality_slope,
            churn_fpr_slope: b.churn_fpr_slope,
            day_slot: (self.day % DAYS_PER_MONTH) as usize,
        }
    }

    fn run_funnel(&mut self, content: &Content, work: &Work, _fx: &OfficeEffects) {
        if work.sales + work.marketing + work.ops > 0.0 || (self.day - 1) % 7 < 5 {
            self.last_work = WorkSummary {
                sales: work.sales,
                marketing: work.marketing,
                ops: work.ops,
                admin: work.admin,
                pitch: work.pitch,
                growth_hackers: work.growth_hackers,
            };
        }
        let mut inp = self.funnel_inputs(content);
        inp.sales_power = work.sales;
        inp.marketing_power = work.marketing;
        inp.support_deficit = self.support_deficit(work.ops, work.sales);
        inp.conv_mul = self.mod_mul(ModKind::ConversionMul) * 0.93f64.powi(work.growth_hackers as i32);
        let res = daily_funnel(&mut self.market, content, &inp, &mut self.rng);
        let new: u32 = res.new.iter().sum();
        let lost: u32 = res.churned.iter().sum();
        for s in Segment::ALL {
            if res.new[s.index()] > 0 && self.market.seg(s).customers == res.new[s.index()] && self.product.launched {
                let flag = format!("first_{:?}", s);
                if !self.flags.contains(&flag) {
                    self.flags.insert(flag);
                    self.log(Tone::Good, Msg::FirstCustomer { segment: s });
                }
            }
        }
        if new > 0 {
            self.cues.push(Cue::NewCustomers(new));
        }
        if lost > 0 {
            self.cues.push(Cue::Churned(lost));
        }
        self.week_new += new;
        self.week_churn += lost;
        self.stats.customers_won += new;
        self.stats.customers_lost += lost;
    }

    fn accrue_money(&mut self, content: &Content) {
        let rr = self.run_rate(content);
        let d = DAYS_PER_MONTH as f64;
        let mrr = self.market.mrr(self.product.launched);
        let m = &mut self.finance.month;
        m.revenue += mrr / d;
        m.payroll += rr.payroll / d;
        m.rent += rr.rent / d;
        m.marketing += rr.marketing / d;
        m.hosting += (rr.hosting + rr.energy) / d;
        m.inference += rr.inference / d;
        m.tools += rr.tools / d;
        m.furniture += rr.furniture / d;
        m.loans += rr.loans / d;
        m.other_income += rr.other_income / d;
        self.finance.cash += (mrr + rr.other_income - rr.expenses()) / d;

        // Masse salariale R&D (base du crédit d'impôt recherche).
        let contrib = content.balancing.employer_contrib;
        let rd: f64 = self
            .employees
            .iter()
            .filter(|e| matches!(e.assignment, Assignment::Task(_)) || e.role == Role::AiResearcher)
            .map(|e| e.monthly_cost(contrib, self.day))
            .sum();
        self.stats.rd_spend += rd / d;

        // Versements.
        let day = self.day;
        let due: Vec<Disbursement> = {
            let (due, keep): (Vec<_>, Vec<_>) =
                self.funding.disbursements.drain(..).partition(|x| x.due_day <= day);
            self.funding.disbursements = keep;
            due
        };
        for x in due {
            self.finance.cash += x.amount;
            match x.kind {
                MoneyKind::Grant => {
                    self.finance.month.grants_in += x.amount;
                    self.finance.total_grants += x.amount;
                    if x.label.contains("Bpi") || x.label.contains("BPI") {
                        self.flags.insert("bpi_certificate".into());
                    }
                }
                MoneyKind::Loan => self.finance.month.loans_in += x.amount,
                MoneyKind::Equity => {
                    self.finance.month.equity_in += x.amount;
                    self.finance.total_equity += x.amount;
                }
                MoneyKind::Income => self.finance.month.other_income += x.amount,
            }
            self.log(Tone::Money, Msg::MoneyIn { label: x.label, amount: x.amount });
            self.cues.push(Cue::Funding(x.amount));
        }
    }

    fn update_people(&mut self, content: &Content, fx: &OfficeEffects, work: &Work, working: bool) {
        let b = &content.balancing;
        let day = self.day;
        let weekend = !working || ((day - 1) % 7 >= 5);
        let tier_morale = content.office(self.office.tier).map(|o| o.morale_bonus).unwrap_or(0.0);
        let growth = self.finance.last_mrr_growth().unwrap_or(0.0);
        let momentum = if growth > 5.0 { 4.0 } else if growth < -5.0 { -6.0 } else { 0.0 };
        let seats = fx.seats as usize;
        let overcrowded = self.employees.len() > seats;
        let hr_bonus = 4.0 * work.hr.min(2) as f64;
        let mod_morale = self.mod_add(ModKind::MoraleTarget);
        let engineers = work.task_workers as f64;
        let equipped = fx.workstations as f64 >= engineers * 0.5;
        let incident_active = self.incident_day.is_some_and(|d| day <= d + 2);
        let base_target = b.morale_base + fx.morale.min(20.0) + tier_morale + momentum + hr_bonus + mod_morale
            - if overcrowded { 10.0 } else { 0.0 }
            - if self.crunch { 8.0 } else { 0.0 };
        let debt_pen = ((self.product.tech_debt - 50.0).max(0.0)) / 5.0;
        let burnout_mult: f64 = if work.hr > 0 { 0.6 } else { 1.0 };

        let mut leaving: Vec<(usize, u8)> = Vec::new(); // 0 démission, 1 fin de contrat
        let mut burnouts = Vec::new();
        for (i, e) in self.employees.iter_mut().enumerate() {
            if let Some(until) = e.on_leave_until
                && day >= until {
                    e.on_leave_until = None;
                    e.fatigue = 20.0;
                }
            let available = e.is_available(day);
            // Fatigue.
            let busy = available && e.assignment != Assignment::Unassigned;
            if !available {
                e.fatigue = (e.fatigue - 3.0).max(0.0);
            } else if weekend && !self.crunch {
                e.fatigue -= b.fatigue_weekend_recovery + e.fatigue * 0.1;
            } else if weekend && self.crunch {
                e.fatigue += b.fatigue_crunch;
            } else if busy {
                let load = if e.assignment == Assignment::Admin { 0.8 } else { 1.0 };
                e.fatigue += b.fatigue_work * load - 0.5 - fx.fatigue_recovery.min(1.2);
            } else {
                e.fatigue -= 3.0;
            }
            e.fatigue = e.fatigue.clamp(0.0, 100.0);
            // Moral.
            let mut target = base_target;
            if e.fatigue > 70.0 {
                target -= (e.fatigue - 70.0) / 3.0;
            }
            if e.role.is_engineering() {
                target -= debt_pen;
            }
            match e.trait_ {
                Trait::ExGafam if !equipped => target -= 8.0,
                Trait::PhdResearcher if matches!(e.assignment, Assignment::Sales | Assignment::Marketing) => {
                    target -= 12.0
                }
                Trait::Founder => target += 8.0,
                _ => {}
            }
            e.morale += (target - e.morale) * b.morale_drift;
            e.morale = e.morale.clamp(0.0, 100.0);
            // Les stagiaires apprennent.
            if e.trait_ == Trait::Stagiaire && busy && !weekend {
                e.skills.eng = (e.skills.eng + 0.05).min(70.0);
                e.skills.research = (e.skills.research + 0.03).min(60.0);
            }
            // Activité pour la couche de présentation.
            e.activity = if !available {
                Activity::OnLeave
            } else if incident_active && e.role.is_engineering() {
                Activity::Panic
            } else if e.fatigue > 85.0 {
                Activity::Exhausted
            } else if e.assignment == Assignment::Unassigned || (weekend && !self.crunch) {
                Activity::Idle
            } else if matches!(e.assignment, Assignment::Management | Assignment::Fundraising | Assignment::Sales) {
                Activity::Meeting
            } else {
                Activity::Working
            };
            if e.role == Role::Founder {
                continue;
            }
            // Burn-out.
            if available && e.fatigue >= 90.0 && self.rng.random_bool((0.06 * burnout_mult).min(1.0)) {
                burnouts.push(i);
                continue;
            }
            // Démission.
            if e.morale < b.resign_morale_threshold {
                e.unhappy_days += 1;
                let p = (0.04 * (1.25 - e.loyalty)).clamp(0.005, 0.05);
                if e.unhappy_days > 10 && self.rng.random_bool(p) {
                    leaving.push((i, 0));
                    continue;
                }
            } else {
                e.unhappy_days = e.unhappy_days.saturating_sub(1);
            }
            if e.contract_end.is_some_and(|d| day >= d) {
                leaving.push((i, 1));
            }
        }
        for i in burnouts {
            let e = &mut self.employees[i];
            e.on_leave_until = Some(day + 21);
            e.fatigue = 40.0;
            let name = e.name.clone();
            self.log(Tone::Bad, Msg::BurnOut { name, days: 21 });
            for o in &mut self.employees {
                o.morale = (o.morale - 3.0).max(0.0);
            }
        }
        // On retire de l'indice le plus haut au plus bas pour que les indices précédents restent valides.
        leaving.sort_by_key(|l| std::cmp::Reverse(l.0));
        for (i, why) in leaving {
            let e = self.employees.remove(i);
            self.cues.push(Cue::Left(e.id));
            if why == 0 {
                self.stats.resigned += 1;
                self.log(Tone::Bad, Msg::Resigned { name: e.name.clone() });
                for o in &mut self.employees {
                    o.morale = (o.morale - 4.0).max(0.0);
                }
            } else {
                let consultant = e.trait_ == Trait::EsnConsultant;
                if consultant {
                    // Le consultant part avec tout le contexte du projet.
                    for t in &mut self.tasks {
                        t.eng_done *= 0.9;
                    }
                    self.product.tech_debt = (self.product.tech_debt + 3.0).min(100.0);
                }
                self.log(Tone::Info, Msg::ContractEnded { name: e.name.clone(), consultant });
            }
        }
        if self.rng.random_bool(0.08) {
            self.stats.coffees += 1;
            self.cues.push(Cue::Coffee);
        }
    }

    fn update_product(&mut self, content: &Content, work: &Work, fx: &OfficeEffects) {
        let b = &content.balancing;
        let level_factor = 1.0 + (self.campaign.level.saturating_sub(1)) as f64 * b.drift_growth_per_level;
        if self.product.has("mvp_text") {
            self.product.drift += b.drift_per_day * level_factor * (1.0 - self.product.robustness);
        }
        let customers = self.market.total_customers() as f64;
        let ops_need = 2.0 + customers / 250.0;
        let ops_ratio = work.ops / ops_need;
        let rust = if work.rust { 0.85 } else { 1.0 };
        let growth = work.task_workers as f64 * b.tech_debt_daily_per_engineer * rust * (1.0 - (ops_ratio * 0.3).min(0.5));
        self.product.tech_debt = (self.product.tech_debt + growth).clamp(0.0, 100.0);
        self.product.incident_penalty = (self.product.incident_penalty - 1.2).max(0.0);
        // Les clients adoptent le produit de plus en plus, jusqu'à un certain point.
        if self.product.launched && self.product.usage_creep < 0.6 {
            let step = b.usage_creep_per_day;
            self.product.usage_creep += step;
            self.product.usage_mult *= 1.0 + step;
        }
        if self.product.launched || customers > 0.0 {
            let p = (b.incident_base + self.product.tech_debt * b.incident_per_debt + customers / 1000.0 * b.incident_per_kcustomer)
                * self.mod_mul(ModKind::IncidentRiskMul)
                * (1.0 + fx.incident_risk)
                * rust
                * (1.0 - self.product.resilience)
                / (1.0 + ops_ratio * 2.0);
            if self.rng.random_bool(p.clamp(0.0, 0.5)) {
                self.trigger_incident(content);
            }
        }
        if self.day % DAYS_PER_MONTH == 14 && self.product.drift > 6.0 {
            let accuracy = self.product.effective_accuracy();
            self.log(Tone::Bad, Msg::DriftWarning { accuracy });
        }
    }

    /// À partir du niveau 10, le produit exige un effectif minimum. En dessous, la dette
    /// technique et la dérive s'accumulent (personne pour les traiter) et l'équipe s'épuise.
    fn apply_staffing(&mut self, content: &Content, working: bool) {
        let Some(need) = content.level(self.campaign.level).and_then(|l| l.staffing) else { return };
        if !working || self.campaign.phase != Phase::Playing {
            return;
        }
        let missing_eng = need.engineers.saturating_sub(self.full_time_engineers()) as f64;
        let headcount = self.employees.len() as u32;
        let missing_staff = need.headcount.saturating_sub(headcount) as f64 / need.headcount.max(1) as f64;
        if missing_eng > 0.0 {
            self.product.tech_debt = (self.product.tech_debt + 0.12 * missing_eng).min(100.0);
            if self.product.has("mvp_text") {
                self.product.drift += 0.03 * missing_eng;
            }
        }
        if missing_staff > 0.0 {
            for e in &mut self.employees {
                e.fatigue = (e.fatigue + 1.2 * missing_staff).min(100.0);
            }
        }
        if (missing_eng > 0.0 || missing_staff > 0.0) && self.day_in_level() % 14 == 7 {
            let ft = self.full_time_engineers();
            self.log(
                Tone::Bad,
                Msg::Text(format!(
                    "Sous-effectif : {headcount}/{} salariés, {ft}/{} ingénieur·es à plein temps. La dette et la fatigue montent.",
                    need.headcount, need.engineers
                )),
            );
        }
    }

    fn update_recruitment(&mut self, content: &Content) {
        let day = self.day;
        let ready: Vec<RecruitmentSearch> = {
            let (ready, keep): (Vec<_>, Vec<_>) = self.searches.drain(..).partition(|s| s.ready_day <= day);
            self.searches = keep;
            ready
        };
        for s in ready {
            let Some(ch) = content.channel(&s.channel) else { continue };
            let pool: Vec<&crate::content::ArchetypeDef> = ch
                .archetypes
                .iter()
                .filter_map(|a| content.archetype(a))
                .filter(|a| a.role == s.role)
                .collect();
            let n = self.rng.random_range(ch.candidates.0..=ch.candidates.1.max(ch.candidates.0));
            if pool.is_empty() || n == 0 {
                self.log(Tone::Bad, Msg::NoCandidates { channel: ch.name.clone() });
                continue;
            }
            for _ in 0..n {
                let arch = pool[self.rng.random_range(0..pool.len())];
                let id = self.new_id();
                let mut rng = self.rng.clone();
                let emp = roll_employee(content, arch, Some(ch), id, day, &mut rng);
                self.rng = rng;
                let hiring_cost = ch.fee_pct * emp.salary * 12.0 + ch.hire_bonus;
                let pitch = if arch.quotes.is_empty() {
                    String::new()
                } else {
                    arch.quotes[self.rng.random_range(0..arch.quotes.len())].clone()
                };
                self.candidates.push(Candidate {
                    id,
                    employee: emp,
                    channel: ch.id.clone(),
                    expires_day: day + 14,
                    hiring_cost,
                    pitch,
                });
            }
            self.log(Tone::Info, Msg::CandidatesArrived { channel: ch.name.clone(), count: n });
        }
        self.candidates.retain(|c| c.expires_day > day);
    }

    fn decay_modifiers(&mut self) {
        for m in &mut self.modifiers {
            m.remaining_days = m.remaining_days.saturating_sub(1);
        }
        self.modifiers.retain(|m| m.remaining_days > 0);
    }

    fn update_events(&mut self, content: &Content) {
        let day = self.day;
        // Conséquences différées.
        let due: Vec<DelayedEffect> = {
            let (due, keep): (Vec<_>, Vec<_>) = self.events.delayed.drain(..).partition(|d| d.due_day <= day);
            self.events.delayed = keep;
            due
        };
        for d in due {
            if self.rng.random_bool(d.chance.clamp(0.0, 1.0)) {
                self.apply_effects(content, &d.effects, &d.source);
                if !d.text.is_empty() {
                    self.log(Tone::Story, Msg::Text(d.text.clone()));
                }
            }
        }
        if self.events.pending.is_some() {
            return;
        }
        if !self.events.queue.is_empty() {
            let next = self.events.queue.remove(0);
            self.fire_event(content, &next.event_id, next.boss);
            return;
        }
        // Événement majeur du niveau.
        if !self.campaign.boss_fired
            && let Some(level) = content.level(self.campaign.level) {
                let trigger = match &level.boss_trigger {
                    crate::content::BossTrigger::Day(d) => self.day_in_level() >= *d,
                    crate::content::BossTrigger::When(c) => self.check(c),
                };
                if trigger {
                    self.campaign.boss_fired = true;
                    let id = level.boss_event.clone();
                    self.fire_event(content, &id, true);
                    return;
                }
            }
        // Événements aléatoires.
        let b = &content.balancing;
        if day < self.events.last_event_day + b.event_min_gap_days || self.day_in_level() < 3 {
            return;
        }
        if !self.rng.random_bool(b.event_chance_per_day) {
            return;
        }
        let candidates: Vec<(usize, f64)> = content
            .events
            .iter()
            .enumerate()
            .filter(|(_, e)| !e.scripted && !self.events.on_cooldown(e, day))
            .filter(|(_, e)| e.conditions.iter().all(|c| self.check(c)))
            .map(|(i, e)| (i, e.weight))
            .collect();
        let total: f64 = candidates.iter().map(|c| c.1).sum();
        if total <= 0.0 {
            return;
        }
        let mut pick = self.rng.random_range(0.0..total);
        for (i, w) in candidates {
            if pick < w {
                let id = content.events[i].id.clone();
                self.fire_event(content, &id, false);
                return;
            }
            pick -= w;
        }
    }

    pub fn fire_event(&mut self, content: &Content, id: &str, boss: bool) {
        let Some(def) = content.event(id) else { return };
        if self.events.pending.is_some() {
            self.events.queue.push(PendingEvent { event_id: id.to_string(), fired_day: self.day, boss });
            return;
        }
        self.events.pending = Some(PendingEvent { event_id: id.to_string(), fired_day: self.day, boss });
        self.events.last_fired.insert(id.to_string(), self.day);
        self.events.fired_once.insert(id.to_string());
        self.events.last_event_day = self.day;
        self.stats.events += 1;
        let immediate = def.immediate.clone();
        self.apply_effects(content, &immediate, &def.title);
        self.cues.push(Cue::EventPopup);
    }

    /// Indique si un choix de l'événement en attente est actuellement possible.
    pub fn choice_allowed(&self, content: &Content, index: usize) -> bool {
        let Some(def) = self.events.pending_def(content) else { return false };
        def.choices.get(index).is_some_and(|c| c.requires.iter().all(|r| self.check(r)))
    }

    fn close_month(&mut self, content: &Content) {
        let month = self.day / DAYS_PER_MONTH;
        let m = std::mem::take(&mut self.finance.month);
        let mrr = self.market.mrr(self.product.launched);
        let customers = self.market.total_customers();
        let new: u32 = self.market.segments.iter().map(|s| s.new_this_month).sum();
        let churned: u32 = self.market.segments.iter().map(|s| s.churned_this_month).sum();
        let start_customers = (customers + churned).saturating_sub(new).max(1);
        let cogs = m.inference + m.hosting;
        let expenses = cogs + m.payroll + m.rent + m.marketing + m.tools + m.furniture + m.loans + m.one_off_costs;
        let report = MonthlyReport {
            month,
            revenue: m.revenue,
            cogs,
            gross_margin: safe_div(m.revenue - cogs, m.revenue),
            payroll: m.payroll,
            rent: m.rent,
            marketing: m.marketing,
            other_opex: m.tools + m.furniture + m.loans,
            one_off_costs: m.one_off_costs,
            net_burn: expenses - m.revenue - m.other_income,
            cash_end: self.finance.cash,
            mrr_end: mrr,
            customers_end: customers,
            new_customers: new,
            churned,
            churn_pct: 100.0 * churned as f64 / start_customers as f64,
            grants_in: m.grants_in,
            equity_in: m.equity_in,
            employees: self.employees.len() as u32,
        };
        let net_burn = report.net_burn;
        self.finance.reports.push(report);
        for s in &mut self.market.segments {
            s.new_this_month = 0;
            s.churned_this_month = 0;
        }
        // Revenus récurrents et prêts.
        let mut ended = Vec::new();
        for inc in &mut self.finance.incomes {
            inc.months_left = inc.months_left.saturating_sub(1);
            if inc.months_left == 0 {
                ended.push(inc.label.clone());
            }
        }
        self.finance.incomes.retain(|i| i.months_left > 0);
        for label in ended {
            self.log(Tone::Info, Msg::IncomeEnded { label });
        }
        let mut repaid = Vec::new();
        for l in &mut self.funding.loans {
            if l.grace_left > 0 {
                l.grace_left -= 1;
            } else if l.months_left > 0 {
                l.months_left -= 1;
                if l.months_left == 0 {
                    repaid.push(l.label.clone());
                }
            }
        }
        self.funding.loans.retain(|l| l.months_left > 0);
        for label in repaid {
            self.log(Tone::Good, Msg::LoanRepaid { label });
        }
        // Dérive du moral des investisseurs et de la concurrence.
        let mc = self.market.market_conditions * self.rng.random_range(0.95..1.06);
        self.market.market_conditions = mc.clamp(0.75, 1.3);
        self.event_pressure *= 0.92;
        self.stats.max_mrr = self.stats.max_mrr.max(mrr);
        self.log(Tone::Info, Msg::MonthClosed { month, net_burn, mrr });
        self.cues.push(Cue::MonthClosed);
        self.recompute_metrics(content);
        if self.metrics.runway < 3.0 {
            let months = self.metrics.runway;
            self.log(Tone::Bad, Msg::LowRunway { months });
            self.cues.push(Cue::Alert);
        }
    }

    fn check_bankruptcy(&mut self, content: &Content) {
        let b = &content.balancing;
        if self.finance.cash >= 0.0 {
            self.finance.negative_since = None;
            return;
        }
        let since = *self.finance.negative_since.get_or_insert(self.day);
        let elapsed = self.day - since;
        if elapsed == 0 || elapsed.is_multiple_of(7) {
            let days_left = b.bankruptcy_grace_days.saturating_sub(elapsed);
            self.log(Tone::Bad, Msg::NegativeCash { days_left });
            self.cues.push(Cue::Alert);
        }
        if elapsed >= b.bankruptcy_grace_days || self.finance.cash < b.bankruptcy_floor {
            self.campaign.phase = Phase::Bankrupt;
            self.cues.push(Cue::Bankrupt);
        }
    }

    // ------------------------------------------------------------------
    // indicateurs dérivés
    // ------------------------------------------------------------------

    pub fn contrib_rate(&self, content: &Content, e: &Employee) -> f64 {
        if self.funding.jei && matches!(e.role, Role::AiResearcher | Role::Founder | Role::Backend) {
            content.balancing.jei_contrib
        } else {
            content.balancing.employer_contrib
        }
    }

    /// Coût d'inférence effectif pour 1 000 analyses (matériel du bureau et
    /// chocs de prix temporaires compris).
    pub fn inference_cost(&self, content: &Content) -> f64 {
        let fx = self.office.effects(content);
        self.product.inference_cost_per_k * (1.0 - fx.inference_cost.min(0.5)) * self.mod_mul(ModKind::InfraCostMul)
    }

    pub fn run_rate(&self, content: &Content) -> RunRate {
        let b = &content.balancing;
        let fx = self.office.effects(content);
        let payroll = self.employees.iter().map(|e| e.monthly_cost(self.contrib_rate(content, e), self.day)).sum();
        let subsidiaries: f64 = self.market.markets.iter().filter_map(|m| content.market(m)).map(|m| m.monthly_cost).sum();
        let rent = content.office(self.office.tier).map(|o| o.rent).unwrap_or(0.0) + self.office.rent_extra + subsidiaries;
        let infra = self.mod_mul(ModKind::InfraCostMul);
        let customers = self.market.total_customers() as f64;
        let hosting = if self.product.has("mvp_text") {
            (b.hosting_base * (1.0 + self.office.tier as f64 * 0.3) + b.hosting_per_customer * customers + self.product.hosting_extra)
                * infra
        } else {
            0.0
        };
        let per_k = self.inference_cost(content);
        let inference: f64 = Segment::ALL
            .iter()
            .map(|&s| {
                self.market.seg(s).customers as f64 * content.segment(s).usage * self.product.usage_mult / 1000.0 * per_k
            })
            .sum();
        let energy = if self.flags.contains("own_datacenters") {
            self.energy_mwh(content) * crate::economy::ENERGY_PRICE_PER_MWH
        } else {
            0.0
        };
        RunRate {
            energy,
            payroll,
            rent,
            marketing: self.market.marketing_budget,
            hosting,
            inference,
            tools: self.employees.len() as f64 * b.tools_per_employee,
            furniture: fx.monthly,
            loans: self.funding.monthly_loan_payments(),
            other_income: self.finance.incomes.iter().map(|i| i.monthly).sum(),
        }
    }

    /// Énergie consommée par mois (MWh) : analyses × kWh pour 1 000 analyses.
    pub fn energy_mwh(&self, content: &Content) -> f64 {
        let analyses_k: f64 = Segment::ALL
            .iter()
            .map(|&s| self.market.seg(s).customers as f64 * content.segment(s).usage * self.product.usage_mult / 1000.0)
            .sum();
        analyses_k * self.product.energy_per_k / 1000.0
    }

    pub fn innovation_score(&self, content: &Content) -> f64 {
        let research = self
            .product
            .shipped
            .iter()
            .filter_map(|t| content.task(t))
            .filter(|t| matches!(t.category, crate::content::TaskCategory::AiResearch | crate::content::TaskCategory::AiNative))
            .count() as f64;
        (research * 8.0 + (self.product.effective_accuracy() - 50.0) * 1.2).clamp(0.0, 100.0)
    }

    pub fn recompute_metrics(&mut self, content: &Content) {
        let b = &content.balancing;
        let rr = self.run_rate(content);
        let mrr = self.market.mrr(self.product.launched);
        let paying = self.paying_customers();
        let gm = if mrr > 0.0 { (mrr - rr.cogs()) / mrr } else { 0.0 };
        let net_burn = rr.expenses() - mrr - rr.other_income;
        let churn = self.market.blended_churn();
        let arpa = safe_div(mrr, paying as f64);
        let new_28: u32 = self.market.segments.iter().map(|s| s.new_last_28()).sum();
        let sales_cost: f64 = self
            .employees
            .iter()
            .filter(|e| matches!(e.assignment, Assignment::Sales | Assignment::Marketing))
            .map(|e| e.monthly_cost(b.employer_contrib, self.day))
            .sum();
        let cac = if new_28 > 0 { (self.market.marketing_budget + sales_cost) / new_28 as f64 } else { 0.0 };
        let ltv_v = ltv(arpa, gm, churn);
        let ltv_cac = if cac > 0.0 { ltv_v / cac } else if ltv_v > 0.0 { 10.0 } else { 0.0 };
        let fx = self.office.effects(content);
        let n = self.employees.len().max(1) as f64;
        let conv_num: f64 = self.market.segments.iter().map(|s| s.conversion * s.leads_per_day).sum();
        let conv_den: f64 = self.market.segments.iter().map(|s| s.leads_per_day).sum();
        let innovation = self.innovation_score(content);
        let engineers = self.employees.iter().filter(|e| e.role.is_engineering()).count() as u32;
        let val = valuation(&ValuationInputs {
            arr: mrr * 12.0,
            growth: self.finance.avg_growth(3),
            gross_margin: gm,
            churn_pct: churn,
            reputation: self.reputation,
            market_conditions: self.market.market_conditions,
            engineers,
            innovation,
            multiple_base: b.valuation_arr_multiple,
            floor: b.valuation_floor,
            per_engineer: b.valuation_per_engineer,
        });
        self.metrics = Metrics {
            mrr,
            arr: mrr * 12.0,
            gross_margin: gm,
            net_burn,
            runway: runway(self.finance.cash, net_burn),
            customers: self.market.total_customers(),
            paying_customers: paying,
            leads: self.market.total_leads(),
            arpa,
            churn,
            conversion: safe_div(conv_num, conv_den),
            cac,
            ltv: ltv_v,
            ltv_cac,
            quality: self.product.quality(),
            accuracy: self.product.effective_accuracy(),
            fpr: self.product.effective_fpr(),
            tech_debt: self.product.tech_debt,
            morale: self.employees.iter().map(|e| e.morale).sum::<f64>() / n,
            fatigue: self.employees.iter().map(|e| e.fatigue).sum::<f64>() / n,
            valuation: val,
            seats: fx.seats,
            employees: self.employees.len() as u32,
            inference_cost_per_k: self.inference_cost(content),
            innovation_score: innovation,
            energy_mwh: self.energy_mwh(content),
            run_rate: rr,
        };
    }

    // ------------------------------------------------------------------
    // commandes
    // ------------------------------------------------------------------

    pub fn apply(&mut self, content: &Content, cmd: Command) -> Result<(), CmdError> {
        let r = self.apply_inner(content, cmd);
        self.recompute_metrics(content);
        if r.is_ok() {
            self.evaluate_missions(content);
        }
        r
    }

    fn apply_inner(&mut self, content: &Content, cmd: Command) -> Result<(), CmdError> {
        let day = self.day;
        match cmd {
            Command::Assign { employee, assignment } => {
                if let Assignment::Task(inst) = assignment
                    && !self.tasks.iter().any(|t| t.instance == inst) {
                        return Err(CmdError::NotFound);
                    }
                let e = self.employees.iter_mut().find(|e| e.id == employee).ok_or(CmdError::NotFound)?;
                e.assignment = assignment;
                Ok(())
            }
            Command::StartTask { task, approach } => {
                if !self.is_unlocked("product_dev") {
                    return Err(CmdError::Locked);
                }
                let def = content.task(&task).ok_or(CmdError::NotFound)?;
                task_availability(def, &self.product, &self.campaign.unlocks, &self.tasks, self.finance.cash)
                    .map_err(|b| match b {
                        crate::research::TaskBlock::NotEnoughCash => CmdError::NotEnoughCash,
                        crate::research::TaskBlock::Locked(_) => CmdError::Locked,
                        crate::research::TaskBlock::MissingDep(d) => {
                            CmdError::Blocked(format!("Prérequis manquant : {}", content.task(&d).map(|t| t.name.as_str()).unwrap_or(&d)))
                        }
                        crate::research::TaskBlock::AlreadyShipped => CmdError::Invalid("Déjà livré"),
                        crate::research::TaskBlock::AlreadyActive => CmdError::Invalid("Déjà en cours"),
                    })?;
                let inst = self.new_id();
                self.finance.cash -= def.cost;
                self.finance.month.one_off_costs += def.cost;
                self.tasks.push(ActiveTask {
                    instance: inst,
                    task_id: def.id.clone(),
                    approach,
                    eng_total: def.eng * approach.effort_mult(),
                    research_total: def.research * approach.effort_mult(),
                    eng_done: 0.0,
                    research_done: 0.0,
                    started_day: day,
                });
                self.log(Tone::Info, Msg::TaskStarted { task: def.name.clone() });
                Ok(())
            }
            Command::CancelTask { instance } => {
                let pos = self.tasks.iter().position(|t| t.instance == instance).ok_or(CmdError::NotFound)?;
                let t = self.tasks.remove(pos);
                let def = content.task(&t.task_id);
                let refund = if t.progress() < 0.05 { def.map(|d| d.cost * 0.5).unwrap_or(0.0) } else { 0.0 };
                self.finance.cash += refund;
                for e in &mut self.employees {
                    if e.assignment == Assignment::Task(instance) {
                        e.assignment = Assignment::Unassigned;
                    }
                }
                let name = def.map(|d| d.name.clone()).unwrap_or_default();
                self.log(Tone::Info, Msg::TaskCancelled { task: name, refund });
                Ok(())
            }
            Command::StartRecruitment { channel, role } => {
                if !self.is_unlocked("recruitment") {
                    return Err(CmdError::Locked);
                }
                let ch = content.channel(&channel).ok_or(CmdError::NotFound)?;
                if !ch.requires.iter().all(|c| self.check(c)) {
                    return Err(CmdError::Invalid("Conditions du canal non remplies"));
                }
                if !ch.archetypes.iter().filter_map(|a| content.archetype(a)).any(|a| a.role == role) {
                    return Err(CmdError::Invalid("Ce canal ne propose pas ce profil"));
                }
                if self.searches.iter().any(|s| s.channel == channel && s.role == role) {
                    return Err(CmdError::Invalid("Recherche déjà en cours"));
                }
                if self.finance.cash < ch.cost {
                    return Err(CmdError::NotEnoughCash);
                }
                self.finance.cash -= ch.cost;
                self.finance.month.one_off_costs += ch.cost;
                let hr = self.employees.iter().any(|e| e.role == Role::Hr && e.assignment == Assignment::Management);
                let mut delay = self.rng.random_range(ch.delay.0..=ch.delay.1.max(ch.delay.0)) as f64;
                if hr {
                    delay *= 0.7;
                }
                let id = self.new_id();
                self.searches.push(RecruitmentSearch {
                    id,
                    channel,
                    role,
                    ready_day: day + delay.round().max(1.0) as u32,
                });
                Ok(())
            }
            Command::Hire { candidate } => {
                if !self.is_unlocked("recruitment") {
                    return Err(CmdError::Locked);
                }
                let pos = self.candidates.iter().position(|c| c.id == candidate).ok_or(CmdError::NotFound)?;
                if self.finance.cash < self.candidates[pos].hiring_cost {
                    return Err(CmdError::NotEnoughCash);
                }
                let c = self.candidates.remove(pos);
                self.finance.cash -= c.hiring_cost;
                self.finance.month.one_off_costs += c.hiring_cost;
                let on_hire = content.channel(&c.channel).map(|ch| ch.on_hire.clone()).unwrap_or_default();
                let mut e = c.employee;
                e.hired_day = day;
                if let Some(arch) = content.archetype(&e.archetype) {
                    e.contract_end = arch.contract_days.map(|d| day + d);
                }
                e.assignment = default_assignment(e.role);
                let (name, role, id) = (e.name.clone(), e.role, e.id);
                self.employees.push(e);
                self.campaign.hired_in_level += 1;
                self.stats.hired += 1;
                self.log(Tone::Good, Msg::Hired { name, role });
                self.cues.push(Cue::Hired(id));
                if !on_hire.is_empty() {
                    self.apply_effects(content, &on_hire, "Recrutement");
                }
                Ok(())
            }
            Command::RejectCandidate { candidate } => {
                self.candidates.retain(|c| c.id != candidate);
                Ok(())
            }
            Command::Fire { employee } => {
                let pos = self.employees.iter().position(|e| e.id == employee).ok_or(CmdError::NotFound)?;
                if self.employees[pos].role == Role::Founder {
                    return Err(CmdError::Invalid("On ne se licencie pas soi-même (enfin, pas encore)"));
                }
                let e = self.employees.remove(pos);
                let years = (day - e.hired_day) as f64 / 336.0;
                let severance = if e.day_rate.is_some() {
                    0.0
                } else {
                    e.salary * (1.0 + years * content.balancing.severance_months_per_year * 4.0)
                };
                self.finance.cash -= severance;
                self.finance.month.one_off_costs += severance;
                self.reputation = (self.reputation - 1.0).max(0.0);
                for o in &mut self.employees {
                    o.morale = (o.morale - 5.0).max(0.0);
                }
                self.stats.fired += 1;
                self.cues.push(Cue::Left(e.id));
                self.log(Tone::Bad, Msg::Fired { name: e.name, severance });
                Ok(())
            }
            Command::SetPrice { segment, price } => {
                let def = content.segment(segment);
                let price = price.clamp(def.min_price, def.max_price);
                let seg = self.market.seg_mut(segment);
                let old = seg.price;
                seg.price = price;
                if price > old * 1.1 && seg.customers > 0 {
                    // Les clients existants sont migrés : certains partent.
                    self.modifiers.push(Modifier {
                        kind: ModKind::ChurnAdd,
                        value: ((price / old) - 1.0) * 6.0,
                        remaining_days: 28,
                        source: "Hausse de prix".into(),
                    });
                }
                self.log(Tone::Info, Msg::PriceChanged { segment, price });
                Ok(())
            }
            Command::SetMarketingBudget(v) => {
                self.market.marketing_budget = v.clamp(0.0, 1_000_000.0);
                Ok(())
            }
            Command::SetCrunch(on) => {
                self.crunch = on;
                Ok(())
            }
            Command::StartApplication { program } => {
                let def = content.program(&program).ok_or(CmdError::NotFound)?;
                if def.requires_unlock.as_ref().is_some_and(|u| !self.is_unlocked(u)) {
                    return Err(CmdError::Locked);
                }
                if !def.requires.iter().all(|c| self.check(c)) {
                    return Err(CmdError::Invalid("Critères d'éligibilité non remplis"));
                }
                if self.funding.active_applications().any(|a| a.program == program) {
                    return Err(CmdError::Invalid("Dossier déjà en cours"));
                }
                if let Some(&last) = self.funding.last_applied.get(&program)
                    && day < last + def.cooldown_days {
                        return Err(CmdError::Invalid("Délai de carence : réessayez plus tard"));
                    }
                if def.cooldown_days == 0 && self.funding.approved.contains(&program) {
                    return Err(CmdError::Invalid("Déjà obtenu"));
                }
                let amount = if def.kind == ProgramKind::TaxCredit {
                    let a = (self.stats.rd_spend * 0.3 / 1000.0).round() * 1000.0;
                    if a < 5_000.0 {
                        return Err(CmdError::Invalid("Pas assez de dépenses de R&D à déclarer"));
                    }
                    self.stats.rd_spend = 0.0;
                    a
                } else {
                    def.amount
                };
                let id = self.new_id();
                self.funding.applications.push(Application {
                    id,
                    program: program.clone(),
                    status: AppStatus::Drafting,
                    admin_done: 0.0,
                    admin_required: def.admin_points,
                    started_day: day,
                    submitted_day: None,
                    decision_day: None,
                    amount,
                    revisions: 0,
                });
                self.funding.last_applied.insert(program, day);
                self.log(Tone::Info, Msg::ApplicationStarted { program: def.name.clone() });
                Ok(())
            }
            Command::StartFundraising { round } => {
                let def = content.round(round).ok_or(CmdError::NotFound)?;
                if def.requires_unlock.as_ref().is_some_and(|u| !self.is_unlocked(u)) {
                    return Err(CmdError::Locked);
                }
                if self.funding.fundraising.is_some() {
                    return Err(CmdError::Invalid("Une levée est déjà en cours"));
                }
                self.funding.fundraising = Some(Fundraising {
                    round,
                    status: RaiseStatus::Pitching,
                    pitch_done: 0.0,
                    pitch_required: def.pitch_points,
                    started_day: day,
                    offers: Vec::new(),
                });
                self.log(Tone::Info, Msg::PitchStarted { round: def.name.clone() });
                Ok(())
            }
            Command::AcceptOffer { index } => {
                let fr = self.funding.fundraising.as_ref().ok_or(CmdError::NotFound)?;
                let offer = fr.offers.get(index).cloned().ok_or(CmdError::NotFound)?;
                if offer.expires_day <= day {
                    return Err(CmdError::Invalid("Offre expirée"));
                }
                let round = fr.round;
                let inv = content.investor(&offer.investor).ok_or(CmdError::NotFound)?;
                let dilution = self.funding.cap.issue(&inv.name, offer.amount, offer.pre_money);
                self.finance.cash += offer.amount;
                self.finance.month.equity_in += offer.amount;
                self.finance.total_equity += offer.amount;
                let level = self.campaign.level;
                *self.funding.equity_by_level.entry(level).or_insert(0.0) += offer.amount;
                self.funding.rounds.push(RoundRecord {
                    kind: round,
                    investor: inv.name.clone(),
                    amount: offer.amount,
                    pre_money: offer.pre_money,
                    dilution,
                    day,
                    level,
                });
                self.funding.fundraising = None;
                let effects = inv.on_accept.clone();
                self.apply_effects(content, &effects, &inv.name);
                self.log(Tone::Story, Msg::Text(inv.accept_line.clone()));
                self.log(Tone::Money, Msg::RoundClosed { investor: inv.name.clone(), amount: offer.amount, dilution });
                self.cues.push(Cue::Funding(offer.amount));
                Ok(())
            }
            Command::DeclineOffers => {
                self.funding.fundraising = None;
                Ok(())
            }
            Command::TakeFriendsFamily => {
                if self.funding.friends_family_done {
                    return Err(CmdError::Invalid("La famille a déjà donné"));
                }
                self.funding.friends_family_done = true;
                let amount = 15_000.0;
                let dilution = self.funding.cap.issue("Tante Martine & les cousins", amount, 185_000.0);
                self.finance.cash += amount;
                self.finance.month.equity_in += amount;
                self.finance.total_equity += amount;
                let level = self.campaign.level;
                *self.funding.equity_by_level.entry(level).or_insert(0.0) += amount;
                self.funding.rounds.push(RoundRecord {
                    kind: RoundKind::FriendsFamily,
                    investor: "Tante Martine & les cousins".into(),
                    amount,
                    pre_money: 185_000.0,
                    dilution,
                    day,
                    level,
                });
                self.log(Tone::Money, Msg::RoundClosed { investor: "Tante Martine & les cousins".into(), amount, dilution });
                self.cues.push(Cue::Funding(amount));
                Ok(())
            }
            Command::PlaceFurniture { item, x, y, rotated } => {
                if !self.is_unlocked("office_custom") {
                    return Err(CmdError::Locked);
                }
                let def = content.furniture(&item).ok_or(CmdError::NotFound)?;
                if def.requires_unlock.as_ref().is_some_and(|u| !self.is_unlocked(u)) {
                    return Err(CmdError::Locked);
                }
                if self.finance.cash < def.cost {
                    return Err(CmdError::NotEnoughCash);
                }
                self.office
                    .place(content, &item, x, y, rotated)
                    .map_err(|e| CmdError::Invalid(e.label()))?;
                self.finance.cash -= def.cost;
                self.finance.month.one_off_costs += def.cost;
                self.log(Tone::Info, Msg::FurniturePlaced { name: def.name.clone() });
                Ok(())
            }
            Command::PlaceStored { index, x, y, rotated } => {
                let item = self.office.storage.get(index).cloned().ok_or(CmdError::NotFound)?;
                self.office
                    .place(content, &item, x, y, rotated)
                    .map_err(|e| CmdError::Invalid(e.label()))?;
                self.office.storage.remove(index);
                Ok(())
            }
            Command::MoveFurniture { uid, x, y, rotated } => {
                if !self.is_unlocked("office_custom") {
                    return Err(CmdError::Locked);
                }
                self.office
                    .relocate(content, uid, x, y, rotated)
                    .map_err(|e| CmdError::Invalid(e.label()))
            }
            Command::SellFurniture { uid } => {
                let pos = self.office.items.iter().position(|i| i.uid == uid).ok_or(CmdError::NotFound)?;
                let it = self.office.items.remove(pos);
                let refund = content.furniture(&it.item).map(|d| d.cost * 0.3).unwrap_or(0.0);
                self.finance.cash += refund;
                Ok(())
            }
            Command::MoveOffice { tier } => {
                let def = content.office(tier).ok_or(CmdError::NotFound)?;
                if tier <= self.office.tier {
                    return Err(CmdError::Invalid("Ce serait un déménagement vers le bas"));
                }
                if def.requires_unlock.as_ref().is_some_and(|u| !self.is_unlocked(u)) {
                    return Err(CmdError::Locked);
                }
                let cost = def.rent * def.deposit_months;
                if self.finance.cash < cost {
                    return Err(CmdError::NotEnoughCash);
                }
                self.finance.cash -= cost;
                self.finance.month.one_off_costs += cost;
                self.office.move_to(content, tier);
                self.reputation = (self.reputation + def.reputation_bonus).min(100.0);
                self.log(Tone::Story, Msg::OfficeMoved { name: def.name.clone() });
                Ok(())
            }
            Command::OpenMarket { market } => {
                if !self.is_unlocked("international") {
                    return Err(CmdError::Locked);
                }
                let def = content.market(&market).ok_or(CmdError::NotFound)?;
                if def.requires_unlock.as_ref().is_some_and(|u| !self.is_unlocked(u)) {
                    return Err(CmdError::Locked);
                }
                if self.market.markets.contains(&market) {
                    return Err(CmdError::Invalid("Marché déjà ouvert"));
                }
                if self.product.languages < def.languages_required {
                    return Err(CmdError::Invalid("Le produit ne parle pas encore cette langue"));
                }
                if self.finance.cash < def.cost {
                    return Err(CmdError::NotEnoughCash);
                }
                self.finance.cash -= def.cost;
                self.finance.month.one_off_costs += def.cost;
                self.market.markets.push(market);
                self.log(Tone::Good, Msg::MarketOpened { name: def.name.clone() });
                Ok(())
            }
            Command::ResolveEvent { choice } => {
                let pending = self.events.pending.clone().ok_or(CmdError::NotFound)?;
                let def = content.event(&pending.event_id).ok_or(CmdError::NotFound)?;
                let ch = def.choices.get(choice).ok_or(CmdError::NotFound)?;
                if !ch.requires.iter().all(|c| self.check(c)) {
                    return Err(CmdError::Invalid("Conditions non remplies pour ce choix"));
                }
                let effects = ch.effects.clone();
                self.apply_effects(content, &effects, &def.title);
                for d in &ch.delayed {
                    self.events.delayed.push(DelayedEffect {
                        due_day: day + d.days,
                        chance: d.chance,
                        effects: d.effects.clone(),
                        text: d.text.clone(),
                        source: def.title.clone(),
                    });
                }
                self.events.history.push(EventRecord { day, event_id: def.id.clone(), choice });
                if !ch.outcome.is_empty() {
                    self.log(Tone::Story, Msg::Text(ch.outcome.clone()));
                }
                self.events.pending = None;
                if !self.events.queue.is_empty() {
                    let next = self.events.queue.remove(0);
                    self.fire_event(content, &next.event_id, next.boss);
                }
                Ok(())
            }
            Command::ChooseMentor { mentor } => self.choose_mentor(content, &mentor),
            Command::ConsultMentor { mentor } => self.consult_mentor(content, &mentor),
            Command::DismissIntro => {
                self.campaign.intro_pending = false;
                Ok(())
            }
            Command::AdvanceLevel => {
                if self.campaign.phase != Phase::LevelComplete {
                    return Err(CmdError::Invalid("Niveau pas encore terminé"));
                }
                let next = self.campaign.level + 1;
                self.start_level(content, next);
                Ok(())
            }
        }
    }
}
