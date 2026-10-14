//! Employees, candidates and recruitment.

use crate::content::{ArchetypeDef, ChannelDef, Content, Trait};
use crate::types::{Role, Seniority};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Skills {
    pub eng: f64,
    pub research: f64,
    pub sales: f64,
    pub marketing: f64,
    pub ops: f64,
}

/// What an employee spends their day on. This *is* the department.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Assignment {
    Unassigned,
    /// Works on an active development task (instance id).
    Task(u32),
    Sales,
    Marketing,
    /// Infrastructure, on-call, tech-debt hygiene.
    Ops,
    /// Paperwork: grant dossiers, France Travail, URSSAF.
    Admin,
    /// Pitch decks and investor meetings.
    Fundraising,
    /// Coordination (PM), people (HR), finance (CFO).
    Management,
}

impl Assignment {
    pub fn label(self) -> &'static str {
        match self {
            Assignment::Unassigned => "Sans affectation",
            Assignment::Task(_) => "Développement",
            Assignment::Sales => "Ventes",
            Assignment::Marketing => "Marketing",
            Assignment::Ops => "Infra / Ops",
            Assignment::Admin => "Administratif",
            Assignment::Fundraising => "Levée de fonds",
            Assignment::Management => "Management",
        }
    }
}

/// High-level state for the presentation layer's animation state machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Activity {
    Idle,
    Working,
    Meeting,
    Exhausted,
    OnLeave,
    Panic,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Employee {
    pub id: u32,
    pub name: String,
    pub role: Role,
    pub archetype: String,
    pub trait_: Trait,
    pub seniority: Seniority,
    /// Gross monthly salary (employer contributions are added on top).
    pub salary: f64,
    /// Daily rate billed by an ESN instead of a salary.
    pub day_rate: Option<f64>,
    pub skills: Skills,
    /// Personal productivity multiplier (0.7..1.3).
    pub productivity: f64,
    pub morale: f64,
    pub fatigue: f64,
    pub loyalty: f64,
    pub assignment: Assignment,
    pub hired_day: u32,
    pub contract_end: Option<u32>,
    pub on_leave_until: Option<u32>,
    /// Fraction of salary covered by an aid for the first 6 months.
    pub subsidy: f64,
    pub activity: Activity,
    /// Days spent below the resignation threshold.
    pub unhappy_days: u32,
    /// Sprite variant for the client (hair/skin palette index).
    pub look: u8,
}

impl Employee {
    pub fn is_available(&self, day: u32) -> bool {
        self.on_leave_until.is_none_or(|d| day >= d)
    }

    /// Monthly cost to the company (salary + contributions, or ESN billing).
    pub fn monthly_cost(&self, employer_contrib: f64, day: u32) -> f64 {
        if let Some(rate) = self.day_rate {
            return rate * 20.0;
        }
        let subsidy = if day < self.hired_day + 168 { self.subsidy } else { 0.0 };
        self.salary * (1.0 + employer_contrib) * (1.0 - subsidy)
    }

    pub fn morale_factor(&self) -> f64 {
        0.5 + self.morale / 100.0
    }

    pub fn fatigue_factor(&self) -> f64 {
        1.0 - ((self.fatigue - 50.0).max(0.0) / 100.0)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Candidate {
    pub id: u32,
    pub employee: Employee,
    pub channel: String,
    pub expires_day: u32,
    /// One-off cost to hire (recruiter fee, referral bonus).
    pub hiring_cost: f64,
    pub pitch: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecruitmentSearch {
    pub id: u32,
    pub channel: String,
    pub role: Role,
    pub ready_day: u32,
}

fn roll(rng: &mut ChaCha8Rng, range: (f64, f64)) -> f64 {
    if range.1 <= range.0 {
        return range.0;
    }
    rng.random_range(range.0..=range.1)
}

pub fn random_name(content: &Content, rng: &mut ChaCha8Rng) -> String {
    let first = &content.names.first[rng.random_range(0..content.names.first.len())];
    let last = &content.names.last[rng.random_range(0..content.names.last.len())];
    format!("{first} {last}")
}

/// Build an employee from an archetype definition.
pub fn roll_employee(
    content: &Content,
    arch: &ArchetypeDef,
    channel: Option<&ChannelDef>,
    id: u32,
    day: u32,
    rng: &mut ChaCha8Rng,
) -> Employee {
    let quality = channel.map(|c| c.quality).unwrap_or(1.0);
    let q = |v: f64| (v * quality).clamp(3.0, 100.0);
    let skills = Skills {
        eng: q(roll(rng, arch.skills.eng)),
        research: q(roll(rng, arch.skills.research)),
        sales: q(roll(rng, arch.skills.sales)),
        marketing: q(roll(rng, arch.skills.marketing)),
        ops: q(roll(rng, arch.skills.ops)),
    };
    let seniority = arch.seniority[rng.random_range(0..arch.seniority.len())];
    let s_frac = match seniority {
        Seniority::Junior => 0.0,
        Seniority::Mid => 0.35,
        Seniority::Senior => 0.7,
        Seniority::Staff => 1.0,
    };
    let salary = (arch.salary.0 + (arch.salary.1 - arch.salary.0) * s_frac)
        * rng.random_range(0.95..1.06);
    Employee {
        id,
        name: random_name(content, rng),
        role: arch.role,
        archetype: arch.id.clone(),
        trait_: arch.trait_,
        seniority,
        salary: (salary / 50.0).round() * 50.0,
        day_rate: arch.day_rate,
        skills,
        productivity: rng.random_range(0.85..1.15),
        morale: rng.random_range(62.0..78.0),
        fatigue: rng.random_range(5.0..20.0),
        loyalty: rng.random_range(0.4..0.9),
        assignment: Assignment::Unassigned,
        hired_day: day,
        contract_end: arch.contract_days.map(|d| day + d),
        on_leave_until: None,
        subsidy: channel.map(|c| c.subsidy).unwrap_or(0.0),
        activity: Activity::Idle,
        unhappy_days: 0,
        look: rng.random_range(0..8),
    }
}

/// Default assignment for a freshly hired employee.
pub fn default_assignment(role: Role) -> Assignment {
    match role {
        Role::Sales => Assignment::Sales,
        Role::Growth => Assignment::Marketing,
        Role::DevOps => Assignment::Ops,
        Role::ProductManager | Role::Hr | Role::Cfo => Assignment::Management,
        _ => Assignment::Unassigned,
    }
}

/// Which assignments make sense for a role (the UI greys out others,
/// the simulation simply yields little output).
pub fn allowed_assignments(role: Role) -> Vec<Assignment> {
    let mut v = vec![Assignment::Unassigned, Assignment::Sales, Assignment::Marketing];
    if role.is_engineering() || role == Role::ProductManager {
        v.push(Assignment::Ops);
    }
    if matches!(role, Role::Founder | Role::Cfo | Role::Hr | Role::ProductManager) {
        v.push(Assignment::Admin);
    }
    if matches!(role, Role::Founder | Role::Cfo) {
        v.push(Assignment::Fundraising);
    }
    if matches!(role, Role::ProductManager | Role::Hr | Role::Cfo) {
        v.push(Assignment::Management);
    }
    v
}
