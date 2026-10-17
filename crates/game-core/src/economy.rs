//! Trésorerie, comptabilité mensuelle et indicateurs SaaS dérivés.
//!
//! Formules (toutes mensuelles) :
//!   MRR           = Σ clients × prix             (0 pendant la bêta gratuite)
//!   ARR           = MRR × 12                     (rythme annualisé, pas un revenu réalisé)
//!   COGS          = inférence + hébergement
//!   Marge brute   = (MRR − COGS) / MRR           (0 quand MRR = 0)
//!   Burn net      = dépenses − MRR − autres revenus
//!   Runway        = trésorerie / burn net        (∞ quand burn net ≤ 0)
//!   LTV           = ARPA × marge brute / churn   (0 quand churn = 0 → plafonnée)
//!   CAC           = (marketing + salaires commerciaux) / nouveaux clients (28 jours)

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MonthAccum {
    pub revenue: f64,
    pub inference: f64,
    pub hosting: f64,
    pub payroll: f64,
    pub rent: f64,
    pub marketing: f64,
    pub tools: f64,
    pub furniture: f64,
    pub loans: f64,
    pub one_off_costs: f64,
    pub grants_in: f64,
    pub equity_in: f64,
    pub loans_in: f64,
    pub other_income: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MonthlyReport {
    pub month: u32,
    pub revenue: f64,
    pub cogs: f64,
    pub gross_margin: f64,
    pub payroll: f64,
    pub rent: f64,
    pub marketing: f64,
    pub other_opex: f64,
    pub one_off_costs: f64,
    pub net_burn: f64,
    pub cash_end: f64,
    pub mrr_end: f64,
    pub customers_end: u32,
    pub new_customers: u32,
    pub churned: u32,
    pub churn_pct: f64,
    pub grants_in: f64,
    pub equity_in: f64,
    pub employees: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Income {
    pub monthly: f64,
    pub months_left: u32,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Finance {
    pub cash: f64,
    pub month: MonthAccum,
    pub reports: Vec<MonthlyReport>,
    /// (jour, trésorerie, mrr) échantillonnés chaque semaine, pour les graphiques.
    pub history: Vec<(u32, f64, f64)>,
    pub negative_since: Option<u32>,
    pub incomes: Vec<Income>,
    pub total_grants: f64,
    pub total_equity: f64,
}

impl Finance {
    pub fn new(cash: f64) -> Self {
        Self {
            cash,
            month: MonthAccum::default(),
            reports: Vec::new(),
            history: vec![(0, cash, 0.0)],
            negative_since: None,
            incomes: Vec::new(),
            total_grants: 0.0,
            total_equity: 0.0,
        }
    }

    /// Croissance du MRR (%) du dernier mois clos par rapport au précédent.
    pub fn last_mrr_growth(&self) -> Option<f64> {
        let n = self.reports.len();
        if n < 2 {
            return None;
        }
        let prev = self.reports[n - 2].mrr_end;
        let cur = self.reports[n - 1].mrr_end;
        if prev <= 0.0 {
            return if cur > 0.0 { Some(100.0) } else { None };
        }
        Some((cur / prev - 1.0) * 100.0)
    }

    /// Croissance mensuelle moyenne du MRR (%) sur les `n` derniers mois
    /// clos, ou None s'il n'y a pas encore assez d'historique.
    pub fn avg_growth_pct(&self, n: usize) -> Option<f64> {
        if self.reports.len() < n + 1 || n == 0 {
            return None;
        }
        let r = &self.reports[self.reports.len() - n - 1..];
        if r[0].mrr_end <= 0.0 {
            return None;
        }
        let total = r[n].mrr_end / r[0].mrr_end;
        Some((total.powf(1.0 / n as f64) - 1.0) * 100.0)
    }

    /// Nombre de mois clos consécutifs (du plus récent) avec une croissance
    /// du MRR d'au moins `min_pct` pour cent.
    pub fn growth_streak(&self, min_pct: f64) -> u32 {
        let mut streak = 0;
        let r = &self.reports;
        for i in (1..r.len()).rev() {
            let prev = r[i - 1].mrr_end;
            let cur = r[i].mrr_end;
            if prev > 0.0 && (cur / prev - 1.0) * 100.0 >= min_pct {
                streak += 1;
            } else {
                break;
            }
        }
        streak
    }

    /// Croissance mensuelle moyenne sur les `n` derniers mois clos (fraction).
    pub fn avg_growth(&self, n: usize) -> f64 {
        let r = &self.reports;
        if r.len() < 2 {
            return 0.0;
        }
        let start = r.len().saturating_sub(n + 1);
        let window = &r[start..];
        let mut sum = 0.0;
        let mut k = 0.0;
        for w in window.windows(2) {
            if w[0].mrr_end > 0.0 {
                sum += (w[1].mrr_end / w[0].mrr_end - 1.0).clamp(-0.5, 1.0);
                k += 1.0;
            }
        }
        if k == 0.0 { 0.0 } else { sum / k }
    }
}

/// Détail du rythme mensuel calculé à partir de l'état courant.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RunRate {
    pub payroll: f64,
    pub rent: f64,
    pub marketing: f64,
    pub hosting: f64,
    pub inference: f64,
    pub tools: f64,
    pub furniture: f64,
    pub loans: f64,
    pub other_income: f64,
    /// Facture d'électricité des datacenters en propre (niveau 16).
    #[serde(default)]
    pub energy: f64,
}

/// Prix de l'électricité pour les datacenters en propre, en € par MWh.
pub const ENERGY_PRICE_PER_MWH: f64 = 180.0;

impl RunRate {
    pub fn cogs(&self) -> f64 {
        self.inference + self.hosting + self.energy
    }
    pub fn expenses(&self) -> f64 {
        self.payroll
            + self.rent
            + self.marketing
            + self.hosting
            + self.inference
            + self.energy
            + self.tools
            + self.furniture
            + self.loans
    }
}

/// Indicateurs dérivés du tableau de bord. Recalculés à chaque tick ; jamais la référence.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Metrics {
    pub mrr: f64,
    pub arr: f64,
    pub run_rate: RunRate,
    pub gross_margin: f64,
    pub net_burn: f64,
    /// Mois ; `f64::INFINITY` quand on ne brûle pas de trésorerie.
    pub runway: f64,
    pub customers: u32,
    pub paying_customers: u32,
    pub leads: f64,
    pub arpa: f64,
    pub churn: f64,
    pub conversion: f64,
    pub cac: f64,
    pub ltv: f64,
    pub ltv_cac: f64,
    pub quality: f64,
    pub accuracy: f64,
    pub fpr: f64,
    pub tech_debt: f64,
    pub morale: f64,
    pub fatigue: f64,
    pub valuation: f64,
    pub seats: u32,
    pub employees: u32,
    pub inference_cost_per_k: f64,
    pub innovation_score: f64,
    /// Énergie consommée par mois, en MWh.
    #[serde(default)]
    pub energy_mwh: f64,
}

pub fn safe_div(a: f64, b: f64) -> f64 {
    if b.abs() < 1e-9 { 0.0 } else { a / b }
}

pub fn runway(cash: f64, net_burn: f64) -> f64 {
    if net_burn <= 0.0 {
        f64::INFINITY
    } else {
        (cash / net_burn).max(0.0)
    }
}

pub fn ltv(arpa: f64, gross_margin: f64, churn_pct: f64) -> f64 {
    if churn_pct <= 0.0 {
        // Plafond à 5 ans de marge quand personne ne part.
        return arpa * gross_margin.max(0.0) * 60.0;
    }
    (arpa * gross_margin.max(0.0) / (churn_pct / 100.0)).min(arpa * 60.0)
}

/// Entrées du modèle de valorisation.
pub struct ValuationInputs {
    pub arr: f64,
    pub growth: f64,
    pub gross_margin: f64,
    pub churn_pct: f64,
    pub reputation: f64,
    pub market_conditions: f64,
    pub engineers: u32,
    pub innovation: f64,
    pub multiple_base: f64,
    pub floor: f64,
    pub per_engineer: f64,
}

/// Valorisation = max(ARR × multiple, plancher équipe/tech) × conditions de marché.
/// multiple = base × croissance × marge × rétention × crédibilité
pub fn valuation(v: &ValuationInputs) -> f64 {
    let growth = (1.0 + v.growth.clamp(-0.3, 0.5) * 8.0).max(0.3);
    let margin = 0.5 + v.gross_margin.clamp(0.0, 1.0) * 0.7;
    let retention = (1.25 - v.churn_pct / 100.0 * 4.0).clamp(0.5, 1.25);
    let credibility = 0.75 + v.reputation.clamp(0.0, 100.0) / 200.0;
    let multiple = v.multiple_base * growth * margin * retention * credibility;
    let floor = v.floor + v.engineers as f64 * v.per_engineer + v.innovation * 15_000.0;
    (v.arr * multiple).max(floor) * v.market_conditions
}
