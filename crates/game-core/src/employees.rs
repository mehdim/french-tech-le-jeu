//! Salariés, candidats et recrutement.

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

/// Ce à quoi un salarié consacre sa journée. C'est *aussi* son département.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Assignment {
    Unassigned,
    /// Travaille sur un chantier de développement actif (id d'instance).
    Task(u32),
    Sales,
    Marketing,
    /// Infrastructure, astreinte, hygiène de la dette technique.
    Ops,
    /// Paperasse : dossiers de subvention, France Travail, URSSAF.
    Admin,
    /// Pitch decks et rendez-vous investisseurs.
    Fundraising,
    /// Coordination (PM), humain (RH), finances (CFO).
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

/// État de haut niveau pour la machine à états d'animation de la couche de présentation.
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
    /// Salaire mensuel brut (les cotisations patronales s'ajoutent par-dessus).
    pub salary: f64,
    /// Taux journalier facturé par une ESN à la place d'un salaire.
    pub day_rate: Option<f64>,
    pub skills: Skills,
    /// Multiplicateur de productivité personnel (0,7..1,3).
    pub productivity: f64,
    pub morale: f64,
    pub fatigue: f64,
    pub loyalty: f64,
    pub assignment: Assignment,
    pub hired_day: u32,
    pub contract_end: Option<u32>,
    pub on_leave_until: Option<u32>,
    /// Part du salaire couverte par une aide pendant les 6 premiers mois.
    pub subsidy: f64,
    pub activity: Activity,
    /// Jours passés sous le seuil de démission.
    pub unhappy_days: u32,
    /// Variante de sprite pour le client (indice de palette cheveux/peau).
    pub look: u8,
}

impl Employee {
    pub fn is_available(&self, day: u32) -> bool {
        self.on_leave_until.is_none_or(|d| day >= d)
    }

    /// Coût mensuel pour l'entreprise (salaire + cotisations, ou facturation ESN).
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
    /// Coût ponctuel d'embauche (frais de cabinet, prime de cooptation).
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

/// Crée un salarié à partir de la définition d'un profil.
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

/// Affectation par défaut d'une nouvelle recrue.
pub fn default_assignment(role: Role) -> Assignment {
    match role {
        Role::Sales => Assignment::Sales,
        Role::Growth => Assignment::Marketing,
        Role::DevOps => Assignment::Ops,
        Role::ProductManager | Role::Hr | Role::Cfo => Assignment::Management,
        _ => Assignment::Unassigned,
    }
}

/// Les affectations qui ont du sens pour un rôle (l'interface grise les autres,
/// la simulation en tire simplement peu de travail).
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
