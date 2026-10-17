//! Application des [`Effect`] à l'état du jeu.

use crate::content::Content;
use crate::economy::Income;
use crate::messages::{Cue, Msg, Tone};
use crate::simulation::GameState;
use crate::types::{Effect, Modifier, Segment};

impl GameState {
    pub fn apply_effects(&mut self, content: &Content, effects: &[Effect], source: &str) {
        for e in effects {
            self.apply_effect(content, e, source);
        }
    }

    pub fn apply_effect(&mut self, content: &Content, effect: &Effect, source: &str) {
        let ceiling = content.balancing.accuracy_ceiling;
        match effect {
            Effect::Cash(v) => {
                self.finance.cash += v;
                if *v >= 0.0 {
                    self.finance.month.other_income += v;
                } else {
                    self.finance.month.one_off_costs += -v;
                }
            }
            Effect::Reputation(v) => self.reputation = (self.reputation + v).clamp(0.0, 100.0),
            Effect::MoraleAll(v) => {
                for e in &mut self.employees {
                    e.morale = (e.morale + v).clamp(0.0, 100.0);
                }
            }
            Effect::FatigueAll(v) => {
                for e in &mut self.employees {
                    e.fatigue = (e.fatigue + v).clamp(0.0, 100.0);
                }
            }
            Effect::MoraleRole(role, v) => {
                for e in self.employees.iter_mut().filter(|e| e.role == *role) {
                    e.morale = (e.morale + v).clamp(0.0, 100.0);
                }
            }
            Effect::TechDebt(v) => {
                self.product.tech_debt = (self.product.tech_debt + v).clamp(0.0, 100.0)
            }
            Effect::Accuracy(v) => {
                if *v >= 0.0 {
                    self.product.gain_accuracy(*v, ceiling);
                } else {
                    self.product.model_accuracy = (self.product.model_accuracy + v).max(50.0);
                }
            }
            Effect::Fpr(v) => self.product.fpr = (self.product.fpr + v).clamp(0.3, 40.0),
            Effect::Drift(v) => self.product.drift = (self.product.drift + v).max(0.0),
            Effect::DriftMul(f) => self.product.drift *= f.max(0.0),
            Effect::Robustness(v) => {
                self.product.robustness = (self.product.robustness + v).clamp(0.0, 0.8)
            }
            Effect::FeatureScore(v) => {
                self.product.feature_score = (self.product.feature_score + v).clamp(0.0, 100.0)
            }
            Effect::Leads(s, v) => {
                let seg = self.market.seg_mut(*s);
                seg.leads = (seg.leads + v).max(0.0);
            }
            Effect::CustomersPct(s, f) => {
                let seg = self.market.seg_mut(*s);
                let delta = (seg.customers as f64 * f).round() as i64;
                seg.customers = (seg.customers as i64 + delta).max(0) as u32;
                if delta < 0 {
                    seg.churned_this_month += (-delta) as u32;
                }
            }
            Effect::AddCustomers(s, n) => {
                let seg = self.market.seg_mut(*s);
                seg.customers += n;
                seg.new_this_month += n;
            }
            Effect::CompetitorPressure(v) => {
                self.market.competitor_pressure = (self.market.competitor_pressure + v).clamp(0.0, 0.9)
            }
            Effect::InferenceCostMul(f) => self.product.inference_cost_per_k *= f.max(0.01),
            Effect::HostingMonthly(v) => {
                self.product.hosting_extra = (self.product.hosting_extra + v).max(0.0)
            }
            Effect::UsageMul(f) => self.product.usage_mult *= f.max(0.01),
            Effect::Retention(s, v) => {
                let r = &mut self.product.retention[s.index()];
                *r = (*r + v).clamp(-0.5, 0.75);
            }
            Effect::ConversionBonus(s, v) => {
                let r = &mut self.product.conversion[s.index()];
                *r = (*r + v).clamp(-0.5, 2.0);
            }
            Effect::Languages(n) => self.product.languages += n,
            Effect::SetFlag(f) => {
                self.flags.insert(f.clone());
            }
            Effect::ClearFlag(f) => {
                self.flags.remove(f);
            }
            Effect::AdminPoints(v) => {
                for a in self.funding.applications.iter_mut() {
                    if a.status == crate::funding::AppStatus::Drafting {
                        a.admin_required += v;
                        a.revisions += 1;
                    }
                }
            }
            Effect::DelayDisbursements(d) => {
                for disb in &mut self.funding.disbursements {
                    disb.due_day += d;
                }
            }
            Effect::Modifier { kind, value, days } => self.modifiers.push(Modifier {
                kind: *kind,
                value: *value,
                remaining_days: *days,
                source: source.to_string(),
            }),
            Effect::Incident => self.trigger_incident(content),
            Effect::QuitRole(role) => {
                if let Some(idx) = self
                    .employees
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.role == *role)
                    .max_by(|a, b| a.1.fatigue.total_cmp(&b.1.fatigue))
                    .map(|(i, _)| i)
                {
                    let e = self.employees.remove(idx);
                    self.log(Tone::Bad, Msg::Resigned { name: e.name });
                    self.cues.push(Cue::Left(e.id));
                }
            }
            Effect::TaskSetback(f) => {
                for t in &mut self.tasks {
                    t.eng_done *= 1.0 - f;
                    t.research_done *= 1.0 - f;
                }
            }
            Effect::Unlock(u) => {
                self.campaign.unlocks.insert(u.clone());
            }
            Effect::SalaryPct(f) => {
                for e in &mut self.employees {
                    if e.salary > 0.0 {
                        e.salary *= 1.0 + f;
                    }
                }
            }
            Effect::FounderProductivity(f) => self.founder_mult *= f,
            Effect::Income { monthly, months, label } => self.finance.incomes.push(Income {
                monthly: *monthly,
                months_left: *months,
                label: label.clone(),
            }),
            Effect::TriggerEvent(id) => self.fire_event(content, id, true),
            Effect::Counter(key, n) => {
                let v = self.counters.entry(key.clone()).or_insert(0);
                *v = (*v as i64 + *n as i64).max(0) as u32;
            }
            Effect::EnergyMul(f) => self.product.energy_per_k *= f.max(0.01),
            Effect::ChooseMentor(id) => {
                let _ = self.choose_mentor(content, id);
            }
            Effect::Resilience(v) => {
                self.product.resilience = (self.product.resilience + v).clamp(0.0, 0.8)
            }
            Effect::RentMonthly(v) => self.office.rent_extra = (self.office.rent_extra + v).max(0.0),
            Effect::PriceAll(f) => {
                for s in Segment::ALL {
                    let def = content.segment(s);
                    let seg = self.market.seg_mut(s);
                    seg.price = (seg.price * (1.0 + f)).clamp(def.min_price, def.max_price);
                }
            }
            Effect::Launch => {
                if !self.product.launched {
                    self.product.launched = true;
                    self.log(Tone::Good, Msg::Launched);
                }
            }
        }
    }

    pub fn trigger_incident(&mut self, _content: &Content) {
        let cost = 500.0 + self.market.total_customers() as f64 * 2.0;
        self.finance.cash -= cost;
        self.finance.month.one_off_costs += cost;
        self.product.incident_penalty = (self.product.incident_penalty + 25.0).min(60.0);
        self.reputation = (self.reputation - 2.0).max(0.0);
        for e in &mut self.employees {
            if e.role.is_engineering() {
                e.fatigue = (e.fatigue + 8.0).min(100.0);
                e.morale = (e.morale - 2.0).max(0.0);
            }
        }
        self.incident_day = Some(self.day);
        self.stats.incidents += 1;
        self.log(Tone::Bad, Msg::Incident { cost });
        self.cues.push(Cue::Incident);
    }
}
