//! Evaluation of [`Condition`]s against the game state.

use crate::simulation::GameState;
use crate::types::{Condition, Segment};

/// Progress toward a condition, for UI bars: (current, target, higher_is_better).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Progress {
    pub current: f64,
    pub target: f64,
    pub higher_is_better: bool,
}

impl Progress {
    fn up(current: f64, target: f64) -> Option<Self> {
        Some(Self { current, target, higher_is_better: true })
    }
    fn down(current: f64, target: f64) -> Option<Self> {
        Some(Self { current, target, higher_is_better: false })
    }
    /// 0..1 fill for a progress bar.
    pub fn fraction(&self) -> f64 {
        if self.higher_is_better {
            if self.target <= 0.0 { 1.0 } else { (self.current / self.target).clamp(0.0, 1.0) }
        } else if self.current <= self.target {
            1.0
        } else {
            (self.target / self.current.max(1e-9)).clamp(0.0, 1.0)
        }
    }
}

impl GameState {
    pub fn day_in_level(&self) -> u32 {
        self.day.saturating_sub(self.campaign.level_start_day)
    }

    pub fn paying_customers(&self) -> u32 {
        if self.product.launched { self.market.total_customers() } else { 0 }
    }

    pub fn check(&self, c: &Condition) -> bool {
        let m = &self.metrics;
        match c {
            Condition::Always => true,
            Condition::CashAtLeast(v) => self.finance.cash >= *v,
            Condition::CashBelow(v) => self.finance.cash < *v,
            Condition::MrrAtLeast(v) => m.mrr >= *v,
            Condition::CustomersAtLeast(n) => self.market.total_customers() >= *n,
            Condition::PayingCustomersAtLeast(n) => self.paying_customers() >= *n,
            Condition::SegmentCustomersAtLeast(s, n) => self.market.seg(*s).customers >= *n,
            Condition::EmployeesAtLeast(n) => self.employees.len() as u32 >= *n,
            Condition::EmployeesBelow(n) => (self.employees.len() as u32) < *n,
            Condition::RoleAtLeast(r, n) => {
                self.employees.iter().filter(|e| e.role == *r).count() as u32 >= *n
            }
            Condition::HiredInLevelAtLeast(n) => self.campaign.hired_in_level >= *n,
            Condition::Shipped(t) => self.product.has(t),
            Condition::NotShipped(t) => !self.product.has(t),
            Condition::ShippedInLevelAtLeast(n) => self.campaign.shipped_in_level >= *n,
            Condition::TaskActive(t) => self.tasks.iter().any(|a| &a.task_id == t),
            Condition::Flag(f) => self.flags.contains(f),
            Condition::NotFlag(f) => !self.flags.contains(f),
            Condition::Unlocked(u) => self.campaign.is_unlocked(u),
            Condition::LevelAtLeast(l) => self.campaign.level >= *l,
            Condition::LevelAtMost(l) => self.campaign.level <= *l,
            Condition::QualityAtLeast(v) => m.quality >= *v,
            Condition::QualityBelow(v) => m.quality < *v,
            Condition::AccuracyAtLeast(v) => m.accuracy >= *v,
            Condition::AccuracyBelow(v) => m.accuracy < *v,
            Condition::FprAtMost(v) => m.fpr <= *v,
            Condition::FprAbove(v) => m.fpr > *v,
            Condition::TechDebtAtLeast(v) => m.tech_debt >= *v,
            Condition::TechDebtBelow(v) => m.tech_debt < *v,
            Condition::MoraleAtLeast(v) => m.morale >= *v,
            Condition::MoraleBelow(v) => m.morale < *v,
            Condition::FatigueAtLeast(v) => m.fatigue >= *v,
            Condition::ReputationAtLeast(v) => self.reputation >= *v,
            Condition::ReputationBelow(v) => self.reputation < *v,
            Condition::ChurnBelow(v) => self.paying_customers() > 0 && m.churn < *v,
            Condition::ChurnAtLeast(v) => self.paying_customers() > 0 && m.churn >= *v,
            Condition::SegmentChurnBelow(s, v) => {
                let seg = self.market.seg(*s);
                seg.customers > 0 && seg.churn_rate < *v
            }
            Condition::RunwayAtLeast(v) => m.runway >= *v,
            Condition::RunwayBelow(v) => m.runway < *v,
            Condition::GrossMarginAtLeast(v) => m.mrr > 0.0 && m.gross_margin * 100.0 >= *v,
            Condition::FundingSecuredInLevelAtLeast(v) => self.secured_in_level() >= *v,
            Condition::ApplicationSubmittedInLevel => self.funding.applications.iter().any(|a| {
                a.submitted_day.is_some_and(|d| d >= self.campaign.level_start_day)
            }),
            Condition::FounderEquityAtLeast(v) => self.funding.cap.founder_pct() >= *v,
            Condition::RoundClosedInLevel { min_amount, max_dilution } => {
                self.funding.rounds.iter().any(|r| {
                    r.level == self.campaign.level
                        && r.amount >= *min_amount
                        && r.dilution <= *max_dilution
                })
            }
            Condition::EquityRaisedInLevelAtLeast(v) => {
                self.funding.equity_by_level.get(&self.campaign.level).copied().unwrap_or(0.0) >= *v
            }
            Condition::MrrGrowthStreak { months, min_pct } => {
                self.finance.avg_growth_pct(*months as usize).is_some_and(|g| g >= *min_pct)
            }
            Condition::InferenceCostReducedBy(f) => {
                let start = self.campaign.snapshot.inference_cost_per_k;
                start > 0.0 && m.inference_cost_per_k <= start * (1.0 - f)
            }
            Condition::CustomersRetainedPct(p) => {
                let start = self.campaign.snapshot.paying_customers as f64;
                start <= 0.0 || self.paying_customers() as f64 >= start * p / 100.0
            }
            Condition::MarketsAtLeast(n) => self.market.markets.len() as u32 >= *n,
            Condition::LanguagesAtLeast(n) => self.product.languages >= *n,
            Condition::LtvCacAtLeast(v) => m.ltv_cac >= *v,
            Condition::NetBurnAtMost(v) => m.net_burn <= *v,
            Condition::ValuationAtLeast(v) => m.valuation >= *v,
            Condition::OfficeTierAtLeast(t) => self.office.tier >= *t,
            Condition::DayInLevelAtLeast(d) => self.day_in_level() >= *d,
            Condition::LeadsAtLeast(v) => m.leads >= *v,
            Condition::Launched => self.product.launched,
            Condition::All(v) => v.iter().all(|c| self.check(c)),
            Condition::Any(v) => v.iter().any(|c| self.check(c)),
            Condition::Not(c) => !self.check(c),
        }
    }

    pub fn secured_in_level(&self) -> f64 {
        let l = self.campaign.level;
        self.funding.secured_by_level.get(&l).copied().unwrap_or(0.0)
            + self.funding.equity_by_level.get(&l).copied().unwrap_or(0.0)
    }

    /// Numeric progress for UI bars, when the condition is quantitative.
    pub fn progress(&self, c: &Condition) -> Option<Progress> {
        let m = &self.metrics;
        match c {
            Condition::CashAtLeast(v) => Progress::up(self.finance.cash, *v),
            Condition::MrrAtLeast(v) => Progress::up(m.mrr, *v),
            Condition::CustomersAtLeast(n) => Progress::up(self.market.total_customers() as f64, *n as f64),
            Condition::PayingCustomersAtLeast(n) => Progress::up(self.paying_customers() as f64, *n as f64),
            Condition::SegmentCustomersAtLeast(s, n) => {
                Progress::up(self.market.seg(*s).customers as f64, *n as f64)
            }
            Condition::EmployeesAtLeast(n) => Progress::up(self.employees.len() as f64, *n as f64),
            Condition::RoleAtLeast(r, n) => Progress::up(
                self.employees.iter().filter(|e| e.role == *r).count() as f64,
                *n as f64,
            ),
            Condition::HiredInLevelAtLeast(n) => Progress::up(self.campaign.hired_in_level as f64, *n as f64),
            Condition::ShippedInLevelAtLeast(n) => {
                Progress::up(self.campaign.shipped_in_level as f64, *n as f64)
            }
            Condition::QualityAtLeast(v) => Progress::up(m.quality, *v),
            Condition::AccuracyAtLeast(v) => Progress::up(m.accuracy, *v),
            Condition::FprAtMost(v) => Progress::down(m.fpr, *v),
            Condition::TechDebtBelow(v) => Progress::down(m.tech_debt, *v),
            Condition::MoraleAtLeast(v) => Progress::up(m.morale, *v),
            Condition::ReputationAtLeast(v) => Progress::up(self.reputation, *v),
            Condition::ChurnBelow(v) => Progress::down(m.churn, *v),
            Condition::SegmentChurnBelow(s, v) => Progress::down(self.market.seg(*s).churn_rate, *v),
            Condition::RunwayAtLeast(v) => Progress::up(m.runway.min(99.0), *v),
            Condition::GrossMarginAtLeast(v) => Progress::up(m.gross_margin * 100.0, *v),
            Condition::FundingSecuredInLevelAtLeast(v) => Progress::up(self.secured_in_level(), *v),
            Condition::FounderEquityAtLeast(v) => Progress::up(self.funding.cap.founder_pct(), *v),
            Condition::EquityRaisedInLevelAtLeast(v) => Progress::up(
                self.funding.equity_by_level.get(&self.campaign.level).copied().unwrap_or(0.0),
                *v,
            ),
            Condition::MrrGrowthStreak { months, min_pct } => {
                Progress::up(self.finance.avg_growth_pct(*months as usize).unwrap_or(0.0).max(0.0), *min_pct)
            }
            Condition::InferenceCostReducedBy(f) => {
                let start = self.campaign.snapshot.inference_cost_per_k;
                let reduced = if start > 0.0 { 1.0 - m.inference_cost_per_k / start } else { 0.0 };
                Progress::up(reduced.max(0.0) * 100.0, f * 100.0)
            }
            Condition::CustomersRetainedPct(p) => {
                let start = self.campaign.snapshot.paying_customers as f64;
                let pct = if start > 0.0 { 100.0 * self.paying_customers() as f64 / start } else { 100.0 };
                Progress::up(pct, *p)
            }
            Condition::MarketsAtLeast(n) => Progress::up(self.market.markets.len() as f64, *n as f64),
            Condition::LanguagesAtLeast(n) => Progress::up(self.product.languages as f64, *n as f64),
            Condition::LtvCacAtLeast(v) => Progress::up(m.ltv_cac, *v),
            Condition::ValuationAtLeast(v) => Progress::up(m.valuation, *v),
            Condition::LeadsAtLeast(v) => Progress::up(m.leads, *v),
            _ => None,
        }
    }

    pub fn segment_unlocked(&self, content: &crate::content::Content, s: Segment) -> bool {
        content
            .segment(s)
            .requires_unlock
            .as_ref()
            .is_none_or(|u| self.campaign.is_unlocked(u))
    }
}
