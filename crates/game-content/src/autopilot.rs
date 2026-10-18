//! Un joueur scripté. Il n'utilise que des [`Command`], exactement ce qu'un humain
//! peut faire depuis l'interface, et sert à :
//!  * le test de campagne (prouve que les seize niveaux sont réalisables),
//!  * le rapport d'équilibrage (`cargo run -p game-content --example balance`),
//!  * le « mode démo » de l'écran titre du client.

use game_core::content::{Content, Trait};
use game_core::employees::Assignment;
use game_core::funding::{AppStatus, RaiseStatus};
use game_core::missions::Phase;
use game_core::research::{Approach, task_availability};
use game_core::simulation::{Command, GameState};
use game_core::types::{Effect, Role, Segment};
use game_core::content::RoundKind;

/// Priorités de développement, dans l'ordre. Les chantiers répétables sont gérés à part.
const TASK_PRIORITY: &[&str] = &[
    "mvp_text", "mvp_dashboard", "eval_dataset", "launch", "mvp_api", "auth", "finetune_v1", "mvp_csv", "self_serve",
    "calibration", "monitoring", "api_keys", "docs", "batch", "integrations", "analytics", "fp_reduction",
    "rate_limit", "media_offer", "edu_offer", "explainability", "churn_alerts", "pricing_lab", "distillation", "inference_infra", "multilingual", "benchmark_v2", "adversarial", "watermark",
    "audit_logs", "sso", "gdpr", "hybrid", "sla", "rbac", "retention_policies", "red_team", "quantization",
    "small_model", "gpu_migration", "sovereign_hosting", "multilingual_2", "provenance", "hitl", "agent_analysis",
    "authenticity_workflow", "trust_platform", "rust_rewrite",
    "soc2", "frontier_eval", "us_region", "zh_language", "watermark_sdk", "zero_trust", "agent_copilot",
    "ipo_readiness", "multi_region", "foundation_detector",
    "model_pruning", "liquid_cooling", "carbon_scheduler", "dc_consolidation", "dc_heat_reuse", "green_ppa",
];

/// Chantiers à passer en tête de file selon le niveau (les objectifs du chapitre d'abord).
fn level_focus(level: u32) -> &'static [&'static str] {
    match level {
        13 => &["soc2"],
        14 => &["zh_language", "watermark_sdk", "zero_trust"],
        15 => &["ipo_readiness"],
        17 => &["crisis_cell", "csrd_report", "explainable_detector", "green_premium", "global_partners"],
        18 => &["kill_sobriety", "close_the_weights", "responsible_compute", "video_8k", "detector_xxl", "cuda_rewrite"],
        16 => &["model_pruning", "liquid_cooling", "carbon_scheduler", "dc_consolidation", "dc_heat_reuse", "green_ppa", "green_premium", "global_partners"],
        _ => &[],
    }
}

/// Marchés que le joueur scripté ouvre, dans l'ordre.
const MARKETS: &[&str] = &["benelux", "de", "es", "it", "nordics", "us", "cn", "in", "jp", "br", "gulf"];

/// Équipe souhaitée (rôle, nombre) par niveau.
fn target_team(level: u32) -> Vec<(Role, usize)> {
    match level {
        1 => vec![],
        2 => vec![(Role::Backend, 1)],
        3 => vec![(Role::Backend, 1), (Role::AiResearcher, 1)],
        4 => vec![(Role::Backend, 1), (Role::AiResearcher, 1), (Role::Sales, 1)],
        5 => vec![(Role::Backend, 2), (Role::AiResearcher, 1), (Role::Sales, 1), (Role::Growth, 1), (Role::DevOps, 1)],
        6 => vec![(Role::Backend, 2), (Role::AiResearcher, 2), (Role::Sales, 2), (Role::Growth, 1), (Role::DevOps, 1)],
        7 => vec![(Role::Backend, 2), (Role::AiResearcher, 3), (Role::Sales, 2), (Role::Growth, 1), (Role::DevOps, 1)],
        8 => vec![(Role::Backend, 3), (Role::AiResearcher, 3), (Role::Sales, 3), (Role::Growth, 1), (Role::DevOps, 1), (Role::Cfo, 1), (Role::ProductManager, 1)],
        9 => vec![(Role::Backend, 4), (Role::AiResearcher, 3), (Role::Sales, 5), (Role::Growth, 2), (Role::DevOps, 2), (Role::Cfo, 1), (Role::ProductManager, 1), (Role::Hr, 1)],
        10 => vec![(Role::Backend, 5), (Role::AiResearcher, 4), (Role::Sales, 7), (Role::Growth, 2), (Role::DevOps, 2), (Role::Cfo, 1), (Role::ProductManager, 1), (Role::Hr, 1)],
        16.. => vec![(Role::Backend, 7), (Role::AiResearcher, 6), (Role::Sales, 20), (Role::Growth, 6), (Role::DevOps, 3), (Role::Cfo, 1), (Role::ProductManager, 2), (Role::Hr, 2)],
        13.. => vec![(Role::Backend, 7), (Role::AiResearcher, 6), (Role::Sales, 12), (Role::Growth, 3), (Role::DevOps, 3), (Role::Cfo, 1), (Role::ProductManager, 2), (Role::Hr, 2)],
        _ => vec![(Role::Backend, 6), (Role::AiResearcher, 5), (Role::Sales, 10), (Role::Growth, 3), (Role::DevOps, 3), (Role::Cfo, 1), (Role::ProductManager, 2), (Role::Hr, 1)],
    }
}

fn channel_for(role: Role, s: &GameState, c: &Content) -> &'static str {
    let rich = s.metrics.runway > 12.0 && s.finance.cash > 150_000.0;
    let ok = |id: &str| c.channel(id).is_some_and(|ch| ch.requires.iter().all(|r| s.check(r)));
    match role {
        Role::AiResearcher => if rich { "recruiter" } else { "ecoles" },
        Role::Backend => if ok("referral") { "referral" } else { "school42" },
        Role::DevOps => if ok("communities") { "communities" } else { "linkedin" },
        Role::Sales => if rich { "recruiter" } else { "linkedin" },
        Role::Cfo => "recruiter",
        _ => "linkedin",
    }
}

fn effect_score(s: &GameState, e: &Effect) -> f64 {
    let poor = s.finance.cash < 40_000.0;
    match e {
        Effect::Cash(v) => v / if poor { 400.0 } else { 3000.0 },
        Effect::Reputation(v) => v * 1.5,
        Effect::MoraleAll(v) => v * 0.6,
        Effect::FatigueAll(v) => -v * 0.3,
        Effect::MoraleRole(_, v) => v * 0.2,
        Effect::TechDebt(v) => -v * 0.4,
        Effect::Accuracy(v) => v * 2.0,
        Effect::Fpr(v) => -v * 3.0,
        Effect::Drift(v) => -v * 1.5,
        Effect::Leads(_, v) => v * 0.05,
        Effect::CustomersPct(_, f) => f * 60.0,
        Effect::AddCustomers(seg, n) => *n as f64 * if *seg == Segment::Enterprise { 6.0 } else { 2.0 },
        Effect::CompetitorPressure(v) => -v * 30.0,
        Effect::InferenceCostMul(f) => (1.0 - f) * 40.0,
        Effect::HostingMonthly(v) => -v / 300.0,
        Effect::Retention(_, v) | Effect::ConversionBonus(_, v) => v * 40.0,
        Effect::Incident => -8.0,
        Effect::QuitRole(_) => -10.0,
        Effect::TaskSetback(f) => -f * 20.0,
        Effect::Resilience(v) => v * 30.0,
        Effect::RentMonthly(v) => -v / 100.0,
        Effect::PriceAll(f) => f * 10.0,
        Effect::ChooseMentor(_) => 1.0,
        Effect::Counter(key, n) => {
            let w = match key.as_str() {
                "state_deals" => 250.0,
                "targeted_laws" | "spy_leaks" => -250.0,
                "cn_oem" => 15.0,
                "stock" => 3.0,
                "gpu_k" => 0.5,
                _ => 1.0,
            };
            w * *n as f64
        }
        Effect::EnergyMul(f) => (1.0 - f) * 40.0,
        Effect::StartTakeover(_) => 100.0,
        Effect::SellTo(_) => -1000.0,
        Effect::SetFlag(f) if f == "nvidia_interest" => 6.0,
        Effect::FounderProductivity(f) => (f - 1.0) * 60.0,
        Effect::Modifier { kind, value, days } => {
            use game_core::types::ModKind::*;
            let d = *days as f64 / 30.0;
            match kind {
                LeadMul | ConversionMul | ProductivityMul | SalesMul | ResearchMul => (value - 1.0) * 20.0 * d,
                InfraCostMul | IncidentRiskMul => (1.0 - value) * 20.0 * d,
                ChurnAdd | CompetitionAdd => -value * 4.0 * d,
                MoraleTarget => value * 0.5 * d,
            }
        }
        Effect::AdminPoints(v) => -v * 0.2,
        _ => 0.0,
    }
}

thread_local! {
    /// Fin que le joueur scripté choisit en priorité (tests des niveaux bonus).
    static ENDING_PREF: std::cell::Cell<Option<&'static str>> = const { std::cell::Cell::new(None) };
}

/// Fixe la fin que le joueur scripté choisira en priorité (`None` = ordre par défaut).
pub fn prefer_ending(ending: Option<&'static str>) {
    ENDING_PREF.with(|p| p.set(ending));
}

fn pick_choice(s: &GameState, c: &Content) -> usize {
    let Some(def) = s.events.pending_def(c) else { return 0 };
    if def.id == "final_decision" {
        let first = ENDING_PREF.with(|p| p.get());
        for pref in first.into_iter().chain(["ending_commons", "ending_profitable", "ending_ipo", "ending_unicorn", "ending_acquisition"]) {
            if let Some(i) = def.choices.iter().position(|ch| {
                ch.effects.iter().any(|e| matches!(e, Effect::SetFlag(f) if f == pref))
            })
                && s.choice_allowed(c, i) {
                    return i;
                }
        }
    }
    let mut best = (f64::MIN, 0);
    for (i, ch) in def.choices.iter().enumerate() {
        if !s.choice_allowed(c, i) {
            continue;
        }
        let mut score: f64 = ch.effects.iter().map(|e| effect_score(s, e)).sum();
        for d in &ch.delayed {
            score += d.chance * 0.8 * d.effects.iter().map(|e| effect_score(s, e)).sum::<f64>();
        }
        if score > best.0 {
            best = (score, i);
        }
    }
    best.1
}

/// Joue une journée : envoie des commandes, puis avance d'un tick. Renvoie false quand la partie est finie.
pub fn step(s: &mut GameState, c: &Content) -> bool {
    match s.campaign.phase {
        Phase::Victory | Phase::Bankrupt => return false,
        Phase::LevelComplete => {
            let _ = s.apply(c, Command::AdvanceLevel);
        }
        Phase::Playing => {}
    }
    if s.campaign.intro_pending {
        let _ = s.apply(c, Command::DismissIntro);
    }
    // Le joueur scripté gagne ses combats contre les patrons des acquisitions.
    if s.takeover.as_ref().is_some_and(|t| t.ready) {
        let _ = s.apply(c, Command::ResolveTakeover { won: true });
    }
    let mut guard = 0;
    while s.events.pending.is_some() && guard < 5 {
        let choice = pick_choice(s, c);
        if s.apply(c, Command::ResolveEvent { choice }).is_err() {
            let _ = s.apply(c, Command::ResolveEvent { choice: 0 });
        }
        guard += 1;
    }
    if s.day.is_multiple_of(3) {
        manage(s, c);
    }
    s.tick(c);
    true
}

/// Consulte chaque mentor dès qu'il est disponible (le joueur scripté écoute ses mentors).
fn consult_mentors(s: &mut GameState, c: &Content) {
    if s.mentors.main.is_none() {
        let _ = s.apply(c, Command::ChooseMentor { mentor: "steeve".into() });
    }
    for m in &c.mentors {
        if s.mentor_wait(m) == 0 && s.mentor_advice(m).is_some() {
            let _ = s.apply(c, Command::ConsultMentor { mentor: m.id.clone() });
        }
    }
}

fn manage(s: &mut GameState, c: &Content) {
    consult_mentors(s, c);
    let level = s.campaign.level;
    let cash = s.finance.cash;
    let burn = s.metrics.net_burn.max(1.0);

    // --- financement --------------------------------------------------------
    if level >= 2 && !s.funding.friends_family_done && (cash < 14_000.0 || s.metrics.runway < 4.0) {
        let _ = s.apply(c, Command::TakeFriendsFamily);
    }
    for p in &c.programs {
        let busy = s.funding.applications.iter().any(|a| a.program == p.id && matches!(a.status, AppStatus::Drafting | AppStatus::Submitted));
        if !busy {
            let _ = s.apply(c, Command::StartApplication { program: p.id.clone() });
        }
    }
    let want_round = match level {
        4 => Some(RoundKind::Angels),
        6 | 7 => if s.metrics.runway < 10.0 || cash < 200_000.0 { Some(RoundKind::Seed) } else { None },
        8 => Some(RoundKind::SeriesA),
        9 | 10 => if s.metrics.runway < 8.0 { Some(RoundKind::SeriesA) } else { None },
        11..=14 => if s.metrics.runway < 10.0 { Some(RoundKind::SeriesB) } else { None },
        15 => Some(RoundKind::SeriesC),
        16 => if s.metrics.runway < 10.0 { Some(RoundKind::SeriesC) } else { None },
        _ => None,
    };
    if let Some(round) = want_round {
        let raised_this_level = s.funding.rounds.iter().any(|r| r.level == level && r.kind == round);
        if s.funding.fundraising.is_none() && !raised_this_level {
            let _ = s.apply(c, Command::StartFundraising { round });
        }
    }
    if let Some(fr) = &s.funding.fundraising
        && fr.status == RaiseStatus::Offers {
            let min_amount = if level == 8 { 3_000_000.0 } else { 0.0 };
            let mut best: Option<(usize, f64)> = None;
            for (i, o) in fr.offers.iter().enumerate() {
                if o.amount < min_amount || o.dilution > 24.5 {
                    continue;
                }
                let score = o.amount / o.dilution.max(1.0);
                if best.is_none_or(|b| score > b.1) {
                    best = Some((i, score));
                }
            }
            match best {
                Some((i, _)) => {
                    let _ = s.apply(c, Command::AcceptOffer { index: i });
                }
                None => {
                    let _ = s.apply(c, Command::DeclineOffers);
                }
            }
        }

    // --- produit --------------------------------------------------------------
    let engineers: Vec<u32> = s
        .employees
        .iter()
        .filter(|e| matches!(e.role, Role::Backend | Role::Frontend | Role::AiResearcher | Role::Intern | Role::Consultant) || (e.role == Role::DevOps && level < 5))
        .map(|e| e.id)
        .collect();
    let max_tasks = (engineers.len() / 2).max(1) + if level <= 1 { 0 } else { 1 };
    let reserve = (burn * 2.0).max(4_000.0);
    if s.product.drift > 4.0 && !s.tasks.iter().any(|t| t.task_id == "retrain") {
        let _ = s.apply(c, Command::StartTask { task: "retrain".into(), approach: Approach::Standard });
    }
    if s.product.tech_debt > 30.0 && !s.tasks.iter().any(|t| t.task_id == "refactor") {
        let _ = s.apply(c, Command::StartTask { task: "refactor".into(), approach: Approach::Standard });
    }
    // Chantiers répétables propres aux derniers chapitres.
    if level >= 14 && s.counter("cn_oem") < 4 && !s.tasks.iter().any(|t| t.task_id == "oem_integration") {
        let _ = s.apply(c, Command::StartTask { task: "oem_integration".into(), approach: Approach::Clean });
    }
    if level == 16 && s.metrics.mrr < 5_000_000.0 && s.product.has("green_premium")
        && !s.tasks.iter().any(|t| t.task_id == "public_tender") {
        let _ = s.apply(c, Command::StartTask { task: "public_tender".into(), approach: Approach::Clean });
    }
    let repeat = |s: &mut GameState, id: &str| {
        if !s.tasks.iter().any(|t| t.task_id == id) {
            let _ = s.apply(c, Command::StartTask { task: id.into(), approach: Approach::Clean });
        }
    };
    if level == 17 {
        if s.counter("stock") < 125 {
            repeat(s, "investor_roadshow");
            repeat(s, "quarterly_results");
        }
        if s.counter("stock") < 105 && s.finance.cash > 15_000_000.0 {
            repeat(s, "share_buyback");
        }
        if s.metrics.mrr < 6_000_000.0 && s.product.has("green_premium") {
            repeat(s, "public_tender");
        }
    }
    if level == 18 {
        if s.counter("gpu_k") < 250 {
            repeat(s, "gpu_bundle");
        }
        if s.metrics.morale < 50.0 {
            repeat(s, "nvidia_bonus");
        }
    }
    if level == 15 && s.counter("state_deals") < 10 && !s.tasks.iter().any(|t| t.task_id == "state_visit") {
        let _ = s.apply(c, Command::StartTask { task: "state_visit".into(), approach: Approach::Clean });
    }
    if s.counter("targeted_laws") >= 1 && !s.product.has("policy_lab") {
        let _ = s.apply(c, Command::StartTask { task: "policy_lab".into(), approach: Approach::Clean });
    }
    for id in level_focus(level).iter().chain(TASK_PRIORITY.iter()) {
        if s.tasks.len() >= max_tasks {
            break;
        }
        let Some(def) = c.task(id) else { continue };
        if task_availability(def, &s.product, &s.campaign.unlocks, &s.tasks, s.finance.cash - reserve).is_ok() {
            let approach = if level <= 2 { Approach::Standard } else { Approach::Clean };
            let _ = s.apply(c, Command::StartTask { task: id.to_string(), approach });
        }
    }

    // --- recrutement ----------------------------------------------------------
    let have = |s: &GameState, r: Role| s.employees.iter().filter(|e| e.role == r).count();
    if s.is_unlocked("recruitment") && s.metrics.runway > 6.0 && cash > 25_000.0 {
        for (role, n) in target_team(level) {
            let pending = s.searches.iter().any(|x| x.role == role) || s.candidates.iter().any(|x| x.employee.role == role);
            if have(s, role) < n && !pending {
                let ch = channel_for(role, s, c);
                let _ = s.apply(c, Command::StartRecruitment { channel: ch.into(), role });
            }
        }
    }
    // Recrutements sur place : San Francisco (niveau 13+), Shanghai (niveau 14+).
    for (channel, market, counter, need, roles) in [
        ("sf_local", "us", "hires_us", 4, [Role::Backend, Role::Sales]),
        ("cn_local", "cn", "hires_cn", 3, [Role::Sales, Role::Sales]),
    ] {
        if !s.market.markets.iter().any(|m| m == market) || s.counter(counter) >= need || s.finance.cash < 200_000.0 {
            continue;
        }
        if !s.searches.iter().any(|x| x.channel == channel) && !s.candidates.iter().any(|x| x.channel == channel) {
            let role = roles[s.counter(counter) as usize % 2];
            let _ = s.apply(c, Command::StartRecruitment { channel: channel.into(), role });
        }
        if let Some(id) = s.candidates.iter().filter(|cd| cd.channel == channel).map(|cd| cd.id).next() {
            let _ = s.apply(c, Command::Hire { candidate: id });
        }
    }
    // Recrute le meilleur candidat pour chaque rôle manquant.
    for (role, n) in target_team(level) {
        if have(s, role) >= n || s.metrics.runway < 5.0 {
            continue;
        }
        let best = s
            .candidates
            .iter()
            .filter(|cd| cd.employee.role == role)
            .max_by(|a, b| {
                let sa = a.employee.skills.eng + a.employee.skills.research + a.employee.skills.sales - a.employee.salary / 200.0;
                let sb = b.employee.skills.eng + b.employee.skills.research + b.employee.skills.sales - b.employee.salary / 200.0;
                sa.total_cmp(&sb)
            })
            .map(|cd| cd.id);
        if let Some(id) = best {
            let _ = s.apply(c, Command::Hire { candidate: id });
        }
    }

    // --- bureau ---------------------------------------------------------------
    let fx = s.office.effects(c);
    let head = s.employees.len() as u32;
    if fx.seats < head + 1 && s.is_unlocked("office_custom") {
        let item = if level >= 5 { "workstation" } else { "desk" };
        if let Some((x, y, r)) = s.office.find_spot(c, item) {
            let _ = s.apply(c, Command::PlaceFurniture { item: item.into(), x, y, rotated: r });
        } else {
            for tier in (s.office.tier + 1)..=5 {
                if s.apply(c, Command::MoveOffice { tier }).is_ok() {
                    break;
                }
            }
        }
    }
    if s.is_unlocked("office_custom") && cash > 20_000.0 {
        for (item, max) in [("coffee_machine", 1usize + head as usize / 8), ("plant", 2 + head as usize / 4), ("sofa", 1), ("whiteboard", 1)] {
            if s.office.count(item) < max.min(c.furniture(item).map(|f| f.max_effective as usize).unwrap_or(1))
                && let Some((x, y, r)) = s.office.find_spot(c, item) {
                    let _ = s.apply(c, Command::PlaceFurniture { item: item.into(), x, y, rotated: r });
                }
        }
        if level >= 10 && s.office.count("gpu_server") < 2 && cash > 300_000.0
            && let Some((x, y, r)) = s.office.find_spot(c, "gpu_server") {
                let _ = s.apply(c, Command::PlaceFurniture { item: "gpu_server".into(), x, y, rotated: r });
            }
    }

    // --- marchés et prix ------------------------------------------------------
    if s.is_unlocked("international") {
        for m in MARKETS {
            let Some(def) = c.market(m) else { continue };
            if s.finance.cash > def.cost + 400_000.0 + def.monthly_cost * 6.0 {
                let _ = s.apply(c, Command::OpenMarket { market: m.to_string() });
            }
        }
    }
    if s.product.launched {
        let mrr = s.metrics.mrr;
        let cap = if level >= 15 { 1_500_000.0 } else { 250_000.0 };

        let budget = if s.metrics.runway < 4.0 { 0.0 } else { (mrr * 0.25).clamp(300.0, cap) };
        let _ = s.apply(c, Command::SetMarketingBudget(budget));
    }

    // --- affectations ---------------------------------------------------------
    assign(s, c);
}

fn assign(s: &mut GameState, c: &Content) {
    let level = s.campaign.level;
    let drafting = s.funding.applications.iter().any(|a| a.status == AppStatus::Drafting);
    let pitching = s.funding.fundraising.as_ref().is_some_and(|f| f.status == RaiseStatus::Pitching)
        || s.takeover.as_ref().is_some_and(|t| !t.ready);
    let has_cfo = s.employees.iter().any(|e| e.role == Role::Cfo);
    let sales_staff = s.employees.iter().filter(|e| e.role == Role::Sales).count();
    let task_ids: Vec<(u32, f64, f64)> = s.tasks.iter().map(|t| (t.instance, t.eng_remaining(), t.research_remaining())).collect();
    let mut cmds = Vec::new();
    let mut eng_i = 0usize;
    let mvp_done = s.product.has("mvp_text");
    let customers = s.market.total_customers();
    for e in &s.employees {
        let want = match e.role {
            Role::Founder => {
                if pitching {
                    Assignment::Fundraising
                } else if drafting && !has_cfo {
                    Assignment::Admin
                } else if !mvp_done || (level <= 1 && customers >= 3) {
                    first_task(&task_ids).unwrap_or(Assignment::Sales)
                } else if (sales_staff == 0 && s.product.has("mvp_text")) || level >= 6 {
                    Assignment::Sales
                } else {
                    first_task(&task_ids).unwrap_or(Assignment::Sales)
                }
            }
            Role::Cfo => {
                if pitching { Assignment::Fundraising } else if drafting { Assignment::Admin } else { Assignment::Management }
            }
            Role::Sales => Assignment::Sales,
            Role::Growth => Assignment::Marketing,
            Role::ProductManager | Role::Hr => Assignment::Management,
            Role::DevOps if level >= 5 || e.trait_ == Trait::RustEvangelist && level >= 4 => Assignment::Ops,
            _ => {
                // Les chercheurs sur les chantiers riches en recherche, les ingénieurs répartis sur les chantiers.
                let pick = if e.role == Role::AiResearcher {
                    task_ids.iter().filter(|t| t.2 > 0.0).max_by(|a, b| a.2.total_cmp(&b.2)).map(|t| Assignment::Task(t.0))
                } else {
                    None
                };
                let fallback = if task_ids.is_empty() {
                    None
                } else {
                    let t = &task_ids[eng_i % task_ids.len()];
                    eng_i += 1;
                    Some(Assignment::Task(t.0))
                };
                pick.or(fallback).unwrap_or(Assignment::Ops)
            }
        };
        if e.assignment != want {
            cmds.push((e.id, want));
        }
    }
    for (id, a) in cmds {
        let _ = s.apply(c, Command::Assign { employee: id, assignment: a });
    }
}

fn first_task(tasks: &[(u32, f64, f64)]) -> Option<Assignment> {
    tasks.iter().find(|t| t.1 > 0.0 || t.2 > 0.0).map(|t| Assignment::Task(t.0))
}

/// Joue une campagne entière. Renvoie l'état final.
pub fn play_campaign(c: &Content, seed: u64, max_days: u32) -> GameState {
    let mut s = GameState::new(c, seed);
    while s.day < max_days && step(&mut s, c) {}
    s
}

/// Comme `play_campaign`, mais en choisissant si possible cette fin (`"ending_ipo"`, …)
/// lors de la décision finale : sert à tester les niveaux bonus.
pub fn play_campaign_choosing(c: &Content, seed: u64, max_days: u32, ending: &'static str) -> GameState {
    ENDING_PREF.with(|p| p.set(Some(ending)));
    let s = play_campaign(c, seed, max_days);
    ENDING_PREF.with(|p| p.set(None));
    s
}
