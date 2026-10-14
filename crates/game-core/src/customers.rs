//! Customer acquisition funnel and churn, per segment.
//!
//! leads/day   = organic(reputation, quality) + paid(budget / CPL) + staff
//! conversion  = base × quality × price × reputation × competition × features
//! churn/month = base × (1 - retention) + quality gap + false positives
//!               + price premium + competition + incidents + modifiers

use crate::content::Content;
use crate::types::{DAYS_PER_MONTH, Segment};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SegmentState {
    pub customers: u32,
    pub price: f64,
    pub leads: f64,
    pub acc_new: f64,
    pub acc_churn: f64,
    /// Projected monthly churn, % (recomputed daily).
    pub churn_rate: f64,
    /// Projected conversion probability of a worked lead.
    pub conversion: f64,
    /// Leads generated per day (last tick).
    pub leads_per_day: f64,
    pub new_this_month: u32,
    pub churned_this_month: u32,
    /// Rolling 28-day windows of daily new / churned customers.
    pub new_window: Vec<u32>,
    pub churn_window: Vec<u32>,
}

impl SegmentState {
    pub fn new(price: f64) -> Self {
        Self {
            customers: 0,
            price,
            leads: 0.0,
            acc_new: 0.0,
            acc_churn: 0.0,
            churn_rate: 0.0,
            conversion: 0.0,
            leads_per_day: 0.0,
            new_this_month: 0,
            churned_this_month: 0,
            new_window: vec![0; DAYS_PER_MONTH as usize],
            churn_window: vec![0; DAYS_PER_MONTH as usize],
        }
    }
    pub fn new_last_28(&self) -> u32 {
        self.new_window.iter().sum()
    }
    pub fn churned_last_28(&self) -> u32 {
        self.churn_window.iter().sum()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketState {
    pub segments: [SegmentState; 3],
    /// Monthly paid-acquisition budget (ads, events, sponsoring).
    pub marketing_budget: f64,
    /// Open geographic markets (France is always open).
    pub markets: Vec<String>,
    /// 0..1, how hard competitors squeeze conversion and churn.
    pub competitor_pressure: f64,
    /// Investor sentiment random walk, multiplies valuations (0.7..1.3).
    pub market_conditions: f64,
}

impl MarketState {
    pub fn new(content: &Content) -> Self {
        Self {
            segments: [
                SegmentState::new(content.segment(Segment::Smb).default_price),
                SegmentState::new(content.segment(Segment::Mid).default_price),
                SegmentState::new(content.segment(Segment::Enterprise).default_price),
            ],
            marketing_budget: 0.0,
            markets: vec!["fr".to_string()],
            competitor_pressure: 0.08,
            market_conditions: 1.0,
        }
    }

    pub fn seg(&self, s: Segment) -> &SegmentState {
        &self.segments[s.index()]
    }
    pub fn seg_mut(&mut self, s: Segment) -> &mut SegmentState {
        &mut self.segments[s.index()]
    }
    pub fn total_customers(&self) -> u32 {
        self.segments.iter().map(|s| s.customers).sum()
    }
    pub fn total_leads(&self) -> f64 {
        self.segments.iter().map(|s| s.leads).sum()
    }
    /// MRR = Σ customers × price, zero while still in free beta.
    pub fn mrr(&self, launched: bool) -> f64 {
        if !launched {
            return 0.0;
        }
        self.segments.iter().map(|s| s.customers as f64 * s.price).sum()
    }
    /// Customer-weighted projected monthly churn, %.
    pub fn blended_churn(&self) -> f64 {
        let total = self.total_customers();
        if total == 0 {
            // Show the SMB projection so the player sees what they'd face.
            return self.segments[0].churn_rate;
        }
        self.segments
            .iter()
            .map(|s| s.churn_rate * s.customers as f64)
            .sum::<f64>()
            / total as f64
    }
}

/// Everything the funnel needs from the rest of the company.
#[derive(Clone, Debug, Default)]
pub struct FunnelInputs {
    pub quality: f64,
    pub fpr: f64,
    pub reputation: f64,
    pub launched: bool,
    pub sales_power: f64,
    pub marketing_power: f64,
    pub lead_mul: f64,
    pub conv_mul: f64,
    pub churn_add: f64,
    pub competition: f64,
    pub unlocked: [bool; 3],
    pub feature_gate: [f64; 3],
    pub retention: [f64; 3],
    pub conversion_bonus: [f64; 3],
    pub market_mult: f64,
    pub incident_recent: f64,
    /// Customers per unit of support/ops capacity above comfort (0 = fine).
    pub support_deficit: f64,
    pub organic_leads: f64,
    pub self_serve_rate: f64,
    pub lead_decay: f64,
    pub churn_quality_slope: f64,
    pub churn_fpr_slope: f64,
    pub day_slot: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FunnelResult {
    pub new: [u32; 3],
    pub churned: [u32; 3],
}

const PAID_SHARE: [f64; 3] = [0.6, 0.3, 0.1];
/// Paid acquisition saturates: at most this many paid leads/day per market.
const PAID_CAP: [f64; 3] = [14.0, 2.5, 0.25];
/// Even a perfect product doesn't convert every lead.
const CONVERSION_CAP: [f64; 3] = [0.40, 0.35, 0.30];

/// Early adopters forgive a lot; mainstream buyers much less.
pub fn quality_factor(quality: f64) -> f64 {
    ((quality + 20.0) / 70.0).clamp(0.3, 1.5)
}

pub fn price_factor(price: f64, reference: f64, elasticity: f64) -> f64 {
    (reference / price.max(1.0)).powf(elasticity).clamp(0.15, 2.5)
}

/// Projected monthly churn for a segment, in %.
pub fn churn_rate(content: &Content, seg: Segment, state: &SegmentState, inp: &FunnelInputs) -> f64 {
    let def = content.segment(seg);
    let i = seg.index();
    let fpr_weight = match seg {
        Segment::Smb => 1.0,
        Segment::Mid => 0.8,
        Segment::Enterprise => 0.55,
    };
    let quality_gap = (60.0 - inp.quality).max(0.0) * inp.churn_quality_slope;
    let fp = inp.fpr * inp.churn_fpr_slope * fpr_weight;
    let price_premium = ((state.price / def.reference_price) - 1.0).max(0.0) * 4.0;
    let competition = inp.competition * 5.0 * if seg == Segment::Enterprise { 0.5 } else { 1.0 };
    let raw = def.base_churn * (1.0 - inp.retention[i]).clamp(0.2, 1.5)
        + (quality_gap + fp) * (1.0 - inp.retention[i] * 0.5).clamp(0.3, 1.0)
        + price_premium
        + competition
        + inp.incident_recent * 2.0
        + inp.support_deficit * 3.0
        + inp.churn_add;
    raw.clamp(0.3, 45.0)
}

/// Lead-to-customer probability for a worked lead.
pub fn conversion_prob(content: &Content, seg: Segment, state: &SegmentState, inp: &FunnelInputs) -> f64 {
    let def = content.segment(seg);
    let i = seg.index();
    let p = def.base_conversion
        * quality_factor(inp.quality)
        * price_factor(state.price, def.reference_price, def.elasticity)
        * (0.7 + inp.reputation / 100.0 * 0.6)
        * (1.0 - inp.competition * 0.6).max(0.2)
        * (0.15 + 0.85 * inp.feature_gate[i])
        * (1.0 + inp.conversion_bonus[i])
        * inp.conv_mul;
    let p = if !inp.launched { p * 1.6 } else { p };
    p.clamp(0.0, CONVERSION_CAP[i])
}

/// Run one simulated day of the funnel.
pub fn daily_funnel(
    market: &mut MarketState,
    content: &Content,
    inp: &FunnelInputs,
    rng: &mut ChaCha8Rng,
) -> FunnelResult {
    let mut result = FunnelResult::default();
    let unlocked_paid: f64 = (0..3).filter(|&i| inp.unlocked[i]).map(|i| PAID_SHARE[i]).sum();
    let budget_daily = market.marketing_budget / DAYS_PER_MONTH as f64;

    // --- 1. leads ------------------------------------------------------
    for seg in Segment::ALL {
        let i = seg.index();
        let def = content.segment(seg);
        let s = &market.segments[i];
        if !inp.unlocked[i] {
            market.segments[i].leads_per_day = 0.0;
            continue;
        }
        let tam = (def.tam_per_market * inp.market_mult).max(1.0);
        let saturation = (1.0 - s.customers as f64 / tam).clamp(0.05, 1.0);
        // Ad auctions get pricier as competitors pile in, and each market
        // only has so many prospects to reach.
        let paid_raw = if unlocked_paid > 0.0 {
            budget_daily * PAID_SHARE[i] / unlocked_paid / (def.cost_per_lead * (1.0 + inp.competition * 2.0))
        } else {
            0.0
        };
        let paid = paid_raw / (1.0 + paid_raw / (PAID_CAP[i] * inp.market_mult));
        let organic = match seg {
            Segment::Smb => {
                inp.organic_leads * (1.0 + inp.reputation / 30.0 + inp.quality / 50.0)
                    + inp.marketing_power * 0.35
            }
            Segment::Mid => {
                inp.organic_leads * 0.12 * (inp.reputation / 10.0)
                    + inp.marketing_power * 0.06
                    + inp.sales_power * 0.03
            }
            Segment::Enterprise => {
                (inp.reputation - 30.0).max(0.0) * 0.004
                    + inp.marketing_power * 0.006
                    + inp.sales_power * 0.006
            }
        };
        let geo = if seg == Segment::Smb { inp.market_mult } else { inp.market_mult.sqrt() };
        let leads = (organic * geo + paid) * inp.lead_mul * saturation;
        let s = &mut market.segments[i];
        s.leads_per_day = leads;
        s.leads += leads;
    }

    // --- 2. sales capacity is shared, weighted by demand ---------------
    let demand: Vec<f64> = Segment::ALL
        .iter()
        .map(|&seg| {
            let i = seg.index();
            if inp.unlocked[i] {
                market.segments[i].leads * content.segment(seg).sales_effort
            } else {
                0.0
            }
        })
        .collect();
    let total_demand: f64 = demand.iter().sum();

    for seg in Segment::ALL {
        let i = seg.index();
        let p = conversion_prob(content, seg, &market.segments[i], inp);
        let churn = churn_rate(content, seg, &market.segments[i], inp);
        let def = content.segment(seg);
        let s = &mut market.segments[i];
        s.conversion = p;
        s.churn_rate = churn;
        if !inp.unlocked[i] {
            s.new_window[inp.day_slot] = 0;
            s.churn_window[inp.day_slot] = 0;
            continue;
        }
        let alloc = if total_demand > 0.0 {
            inp.sales_power * demand[i] / total_demand
        } else {
            0.0
        };
        let worked = (alloc / def.sales_effort).min(s.leads);
        let self_serve = if seg == Segment::Smb { (s.leads - worked).max(0.0) * inp.self_serve_rate } else { 0.0 };
        let expected = worked * p + self_serve * p * 0.6;
        s.leads = (s.leads - worked - self_serve).max(0.0) * (1.0 - inp.lead_decay);
        // A little noise so days don't feel scripted.
        s.acc_new += expected * rng.random_range(0.8..1.2);
        let new = s.acc_new.floor() as u32;
        s.acc_new -= new as f64;

        let churn_expected = s.customers as f64 * churn / 100.0 / DAYS_PER_MONTH as f64;
        s.acc_churn += churn_expected * rng.random_range(0.75..1.25);
        let lost = (s.acc_churn.floor() as u32).min(s.customers + new);
        s.acc_churn -= lost as f64;

        s.customers = s.customers + new - lost;
        s.new_this_month += new;
        s.churned_this_month += lost;
        s.new_window[inp.day_slot] = new;
        s.churn_window[inp.day_slot] = lost;
        result.new[i] = new;
        result.churned[i] = lost;
    }
    result
}

/// Components of a segment's projected churn, for the UI (label, points).
pub fn churn_breakdown(content: &Content, seg: Segment, state: &SegmentState, inp: &FunnelInputs) -> Vec<(&'static str, f64)> {
    let def = content.segment(seg);
    let i = seg.index();
    let fpr_weight = match seg {
        Segment::Smb => 1.0,
        Segment::Mid => 0.8,
        Segment::Enterprise => 0.55,
    };
    let damp = (1.0 - inp.retention[i] * 0.5).clamp(0.3, 1.0);
    vec![
        ("Base du segment (après rétention produit)", def.base_churn * (1.0 - inp.retention[i]).clamp(0.2, 1.5)),
        ("Qualité insuffisante (< 60)", (60.0 - inp.quality).max(0.0) * inp.churn_quality_slope * damp),
        ("Faux positifs", inp.fpr * inp.churn_fpr_slope * fpr_weight * damp),
        ("Prix au-dessus de la référence", ((state.price / def.reference_price) - 1.0).max(0.0) * 4.0),
        ("Concurrence", inp.competition * 5.0 * if seg == Segment::Enterprise { 0.5 } else { 1.0 }),
        ("Incidents récents", inp.incident_recent * 2.0),
        ("Support débordé", inp.support_deficit * 3.0),
        ("Événements en cours", inp.churn_add),
    ]
}
