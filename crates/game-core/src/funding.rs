//! Financement public (France Travail, Bpifrance, CIR, JEI), prêts et levées de fonds.

use crate::content::RoundKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppStatus {
    /// Paperasse en cours : les points d'administratif s'accumulent.
    Drafting,
    /// En attente du comité.
    Submitted,
    Approved,
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Application {
    pub id: u32,
    pub program: String,
    pub status: AppStatus,
    pub admin_done: f64,
    pub admin_required: f64,
    pub started_day: u32,
    pub submitted_day: Option<u32>,
    pub decision_day: Option<u32>,
    pub amount: f64,
    /// Nombre de fois où le dossier a été renvoyé pour des pièces supplémentaires.
    pub revisions: u32,
}

impl Application {
    pub fn progress(&self) -> f64 {
        if self.admin_required <= 0.0 {
            1.0
        } else {
            (self.admin_done / self.admin_required).clamp(0.0, 1.0)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoneyKind {
    Grant,
    Loan,
    Equity,
    Income,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Disbursement {
    pub due_day: u32,
    pub amount: f64,
    pub label: String,
    pub kind: MoneyKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Loan {
    pub label: String,
    pub monthly: f64,
    /// Mois de différé avant le début des remboursements.
    pub grace_left: u32,
    pub months_left: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CapTable {
    pub founder: f64,
    pub holders: Vec<(String, f64)>,
}

impl CapTable {
    pub fn total(&self) -> f64 {
        self.founder + self.holders.iter().map(|h| h.1).sum::<f64>()
    }
    pub fn founder_pct(&self) -> f64 {
        100.0 * self.founder / self.total()
    }
    /// Émet de nouvelles actions pour `amount` à `pre_money` ; renvoie la dilution en %.
    pub fn issue(&mut self, holder: &str, amount: f64, pre_money: f64) -> f64 {
        let total = self.total();
        let new_shares = total * amount / pre_money.max(1.0);
        if let Some(h) = self.holders.iter_mut().find(|h| h.0 == holder) {
            h.1 += new_shares;
        } else {
            self.holders.push((holder.to_string(), new_shares));
        }
        100.0 * new_shares / (total + new_shares)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Offer {
    pub investor: String,
    pub amount: f64,
    pub pre_money: f64,
    pub dilution: f64,
    pub expires_day: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RaiseStatus {
    Pitching,
    Offers,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fundraising {
    pub round: RoundKind,
    pub status: RaiseStatus,
    pub pitch_done: f64,
    pub pitch_required: f64,
    pub started_day: u32,
    pub offers: Vec<Offer>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoundRecord {
    pub kind: RoundKind,
    pub investor: String,
    pub amount: f64,
    pub pre_money: f64,
    pub dilution: f64,
    pub day: u32,
    pub level: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FundingState {
    pub applications: Vec<Application>,
    pub disbursements: Vec<Disbursement>,
    pub loans: Vec<Loan>,
    pub cap: CapTable,
    pub fundraising: Option<Fundraising>,
    pub rounds: Vec<RoundRecord>,
    /// id du dispositif -> jour du dernier dépôt (délais de carence).
    pub last_applied: BTreeMap<String, u32>,
    /// Dispositifs accordés au moins une fois.
    pub approved: Vec<String>,
    /// Argent obtenu (accordé ou encaissé) par niveau, pour les objectifs.
    pub secured_by_level: BTreeMap<u32, f64>,
    pub equity_by_level: BTreeMap<u32, f64>,
    pub jei: bool,
    pub friends_family_done: bool,
}

impl FundingState {
    pub fn new() -> Self {
        Self {
            applications: Vec::new(),
            disbursements: Vec::new(),
            loans: Vec::new(),
            cap: CapTable { founder: 1_000_000.0, holders: Vec::new() },
            fundraising: None,
            rounds: Vec::new(),
            last_applied: BTreeMap::new(),
            approved: Vec::new(),
            secured_by_level: BTreeMap::new(),
            equity_by_level: BTreeMap::new(),
            jei: false,
            friends_family_done: false,
        }
    }

    pub fn active_applications(&self) -> impl Iterator<Item = &Application> {
        self.applications
            .iter()
            .filter(|a| matches!(a.status, AppStatus::Drafting | AppStatus::Submitted))
    }

    pub fn monthly_loan_payments(&self) -> f64 {
        self.loans
            .iter()
            .filter(|l| l.grace_left == 0 && l.months_left > 0)
            .map(|l| l.monthly)
            .sum()
    }

    pub fn secure(&mut self, level: u32, amount: f64) {
        *self.secured_by_level.entry(level).or_insert(0.0) += amount;
    }
}

impl Default for FundingState {
    fn default() -> Self {
        Self::new()
    }
}
