//! Définitions chargées depuis les fichiers de données. La simulation ne fait que les
//! *lire* ; tout ce qui change vit dans [`crate::simulation::GameState`].

use crate::balancing::Balancing;
use crate::types::{Condition, Effect, Role, Segment, Seniority};
use serde::{Deserialize, Serialize};

/// Une réplique dans une scène de l'histoire.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Line {
    pub speaker: String,
    pub text: String,
}

/// Catégorie d'un chantier de développement (branche de l'arbre produit).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskCategory {
    Mvp,
    Engineering,
    AiResearch,
    Enterprise,
    AiNative,
    Maintenance,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: TaskCategory,
    pub eng: f64,
    pub research: f64,
    pub cost: f64,
    #[serde(default)]
    pub deps: Vec<String>,
    #[serde(default)]
    pub requires_unlock: Option<String>,
    pub effects: Vec<Effect>,
    /// Dette technique ajoutée à la livraison (approche Standard).
    #[serde(default)]
    pub debt: f64,
    /// Probabilité (0..1) d'une complication à la livraison.
    #[serde(default)]
    pub risk: f64,
    /// Peut être relancé après livraison (chantiers de maintenance).
    #[serde(default)]
    pub repeatable: bool,
    /// Compte pour « livrer N améliorations significatives ».
    #[serde(default)]
    pub meaningful: bool,
    /// Court texte « bénéfice attendu » pour l'interface.
    #[serde(default)]
    pub benefit: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillRange {
    pub eng: (f64, f64),
    pub research: (f64, f64),
    pub sales: (f64, f64),
    pub marketing: (f64, f64),
    pub ops: (f64, f64),
}

/// Traits de personnalité avec des conséquences mécaniques (voir `employees.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Trait {
    Standard,
    Polytechnicien,
    Dev42,
    RustEvangelist,
    ExGafam,
    EsnConsultant,
    GrowthHacker,
    PhdResearcher,
    JiraMaster,
    Stagiaire,
    Founder,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchetypeDef {
    pub id: String,
    pub name: String,
    pub role: Role,
    pub trait_: Trait,
    pub description: String,
    pub skills: SkillRange,
    pub salary: (f64, f64),
    pub seniority: Vec<Seniority>,
    /// Durée de contrat fixe en jours (consultants, stagiaires).
    #[serde(default)]
    pub contract_days: Option<u32>,
    /// Taux journalier facturé à la place d'un salaire (consultants d'ESN).
    #[serde(default)]
    pub day_rate: Option<f64>,
    #[serde(default)]
    pub quotes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: f64,
    /// Frais en fraction du salaire brut annuel de la recrue (cabinets de recrutement).
    #[serde(default)]
    pub fee_pct: f64,
    pub delay: (u32, u32),
    pub candidates: (u32, u32),
    /// Identifiants des profils que ce canal peut proposer.
    pub archetypes: Vec<String>,
    #[serde(default)]
    pub requires: Vec<Condition>,
    /// Part du salaire subventionnée pendant les premiers mois (aide France Travail).
    #[serde(default)]
    pub subsidy: f64,
    /// Prime versée à l'embauche (prime de cooptation).
    #[serde(default)]
    pub hire_bonus: f64,
    /// Multiplicateur de qualité sur les compétences tirées.
    #[serde(default = "one")]
    pub quality: f64,
}

fn one() -> f64 {
    1.0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FloorStyle {
    Parquet,
    Carpet,
    Concrete,
    Tiles,
    Wood,
    Marble,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OfficeTierDef {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub width: i32,
    pub height: i32,
    pub rent: f64,
    pub deposit_months: f64,
    pub requires_unlock: Option<String>,
    pub floor: FloorStyle,
    /// Coût de base d'hébergement/ops ajouté par une infrastructure plus grande.
    #[serde(default)]
    pub morale_bonus: f64,
    #[serde(default)]
    pub reputation_bonus: f64,
    /// Décorations murales affichées dans ce bureau.
    #[serde(default)]
    pub wall_decor: Vec<String>,
    /// Mobilier offert à l'emménagement (agencement x, y, objet).
    #[serde(default)]
    pub starter: Vec<(i32, i32, String)>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FurnitureEffects {
    #[serde(default)]
    pub seats: u32,
    #[serde(default)]
    pub morale: f64,
    #[serde(default)]
    pub fatigue_recovery: f64,
    #[serde(default)]
    pub eng_bonus: f64,
    #[serde(default)]
    pub research_bonus: f64,
    #[serde(default)]
    pub sales_bonus: f64,
    #[serde(default)]
    pub incident_risk: f64,
    #[serde(default)]
    pub inference_cost: f64,
    #[serde(default)]
    pub productivity: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FurnitureDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: f64,
    #[serde(default)]
    pub monthly: f64,
    pub size: (i32, i32),
    pub effects: FurnitureEffects,
    /// Rendement décroissant : seuls ce nombre d'exemplaires comptent pleinement.
    #[serde(default = "max_eff")]
    pub max_effective: u32,
    #[serde(default)]
    pub requires_unlock: Option<String>,
    /// Nom du sprite dans l'atlas du client.
    pub sprite: String,
}

fn max_eff() -> u32 {
    99
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProgramKind {
    Grant,
    /// Moitié subvention, moitié avance remboursable.
    Advance,
    Loan,
    /// Crédit d'impôt recherche calculé sur la masse salariale de recherche.
    TaxCredit,
    /// Un statut qui réduit les cotisations patronales du personnel R&D.
    Status,
    /// Accompagnement des créateurs (France Travail).
    FounderSupport,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProgramDef {
    pub id: String,
    pub name: String,
    pub institution: String,
    pub description: String,
    pub kind: ProgramKind,
    pub requires_unlock: Option<String>,
    #[serde(default)]
    pub requires: Vec<Condition>,
    pub admin_points: f64,
    pub wait_days: u32,
    pub amount: f64,
    /// (délai après l'accord en jours, fraction du montant)
    pub tranches: Vec<(u32, f64)>,
    pub base_success: f64,
    pub cooldown_days: u32,
    /// Prêts/avances : remboursements mensuels après un différé.
    #[serde(default)]
    pub repay_months: u32,
    #[serde(default)]
    pub grace_months: u32,
    #[serde(default)]
    pub on_approve: Vec<Effect>,
    #[serde(default)]
    pub approve_text: String,
    #[serde(default)]
    pub reject_text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RoundKind {
    FriendsFamily,
    Angels,
    Seed,
    SeriesA,
    SeriesB,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvestorDef {
    pub id: String,
    pub name: String,
    pub firm: String,
    pub bio: String,
    pub rounds: Vec<RoundKind>,
    /// Fourchette du ticket en euros.
    pub ticket: (f64, f64),
    /// Multiplicateur de valorisation par rapport au modèle de marché (<1 = exigeant).
    pub valuation_mult: f64,
    /// Indicateur qui compte pour cet investisseur (affiché dans l'interface).
    pub favourite_metric: String,
    /// Conditions pour que cet investisseur fasse une offre.
    #[serde(default)]
    pub requires: Vec<Condition>,
    /// Appliqué quand le joueur accepte l'offre.
    #[serde(default)]
    pub on_accept: Vec<Effect>,
    pub pitch_line: String,
    pub accept_line: String,
    pub reject_line: String,
    /// Exigence de l'investisseur affichée avec l'offre.
    #[serde(default)]
    pub demand: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoundDef {
    pub kind: RoundKind,
    pub name: String,
    pub requires_unlock: Option<String>,
    pub pitch_points: f64,
    /// Fourchette de dilution typique visée par les investisseurs, en %.
    pub dilution: (f64, f64),
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventChoice {
    pub label: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub requires: Vec<Condition>,
    #[serde(default)]
    pub effects: Vec<Effect>,
    #[serde(default)]
    pub delayed: Vec<DelayedDef>,
    /// Texte affiché après le choix.
    #[serde(default)]
    pub outcome: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DelayedDef {
    pub days: u32,
    pub chance: f64,
    pub effects: Vec<Effect>,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventDef {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub speaker: String,
    #[serde(default)]
    pub conditions: Vec<Condition>,
    #[serde(default = "one")]
    pub weight: f64,
    #[serde(default)]
    pub cooldown: u32,
    /// Ne peut se déclencher qu'une fois par campagne.
    #[serde(default)]
    pub once: bool,
    /// Les événements majeurs/scénarisés ne sont jamais tirés au hasard.
    #[serde(default)]
    pub scripted: bool,
    pub choices: Vec<EventChoice>,
    /// Effets appliqués dès que l'événement se déclenche (avant le choix).
    #[serde(default)]
    pub immediate: Vec<Effect>,
    /// Clé de sprite/illustration pour le client.
    #[serde(default)]
    pub art: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestKind {
    /// Reste acquis une fois atteint.
    Reach,
    /// Doit être vrai au moment où le niveau est évalué.
    Maintain,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuestDef {
    pub id: String,
    pub label: String,
    pub kind: QuestKind,
    pub condition: Condition,
    #[serde(default)]
    pub hint: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BossTrigger {
    Day(u32),
    When(Condition),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LevelDef {
    pub id: u32,
    pub title: String,
    pub subtitle: String,
    pub objective: String,
    pub intro: Vec<Line>,
    pub quests: Vec<QuestDef>,
    pub unlocks: Vec<String>,
    pub unlock_text: Vec<String>,
    pub boss_event: String,
    pub boss_trigger: BossTrigger,
    #[serde(default)]
    pub start_effects: Vec<Effect>,
    pub failure_risk: String,
    pub outro: Vec<Line>,
    /// Niveau de bureau vers lequel l'histoire vous fait déménager au début de ce niveau (s'il est inférieur).
    #[serde(default)]
    pub min_office: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarketDef {
    pub id: String,
    pub name: String,
    pub flag: String,
    pub cost: f64,
    pub languages_required: u32,
    pub lead_mult: f64,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EndingDef {
    pub id: String,
    pub title: String,
    pub requires: Vec<Condition>,
    pub requirement_text: String,
    pub epilogue: Vec<Line>,
    pub headline: String,
    pub art: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NamePools {
    pub first: Vec<String>,
    pub last: Vec<String>,
}

/// Prix par défaut et profil du segment.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SegmentDef {
    pub segment: Segment,
    pub name: String,
    pub default_price: f64,
    pub reference_price: f64,
    pub min_price: f64,
    pub max_price: f64,
    pub elasticity: f64,
    pub base_conversion: f64,
    pub sales_effort: f64,
    pub base_churn: f64,
    /// Analyses mensuelles par client.
    pub usage: f64,
    pub cost_per_lead: f64,
    pub requires_unlock: Option<String>,
    /// Fonctionnalités livrées nécessaires pour bien vendre dans ce segment.
    pub required_features: Vec<String>,
    pub tam_per_market: f64,
}

/// Tout le contenu statique du jeu.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Content {
    pub balancing: Balancing,
    pub tasks: Vec<TaskDef>,
    pub archetypes: Vec<ArchetypeDef>,
    pub channels: Vec<ChannelDef>,
    pub offices: Vec<OfficeTierDef>,
    pub furniture: Vec<FurnitureDef>,
    pub programs: Vec<ProgramDef>,
    pub investors: Vec<InvestorDef>,
    pub rounds: Vec<RoundDef>,
    pub events: Vec<EventDef>,
    pub levels: Vec<LevelDef>,
    pub markets: Vec<MarketDef>,
    pub endings: Vec<EndingDef>,
    pub segments: Vec<SegmentDef>,
    pub names: NamePools,
}

impl Content {
    pub fn task(&self, id: &str) -> Option<&TaskDef> {
        self.tasks.iter().find(|t| t.id == id)
    }
    pub fn archetype(&self, id: &str) -> Option<&ArchetypeDef> {
        self.archetypes.iter().find(|a| a.id == id)
    }
    pub fn channel(&self, id: &str) -> Option<&ChannelDef> {
        self.channels.iter().find(|c| c.id == id)
    }
    pub fn office(&self, id: u32) -> Option<&OfficeTierDef> {
        self.offices.iter().find(|o| o.id == id)
    }
    pub fn furniture(&self, id: &str) -> Option<&FurnitureDef> {
        self.furniture.iter().find(|f| f.id == id)
    }
    pub fn program(&self, id: &str) -> Option<&ProgramDef> {
        self.programs.iter().find(|p| p.id == id)
    }
    pub fn investor(&self, id: &str) -> Option<&InvestorDef> {
        self.investors.iter().find(|p| p.id == id)
    }
    pub fn round(&self, kind: RoundKind) -> Option<&RoundDef> {
        self.rounds.iter().find(|r| r.kind == kind)
    }
    pub fn event(&self, id: &str) -> Option<&EventDef> {
        self.events.iter().find(|e| e.id == id)
    }
    pub fn level(&self, id: u32) -> Option<&LevelDef> {
        self.levels.iter().find(|l| l.id == id)
    }
    pub fn market(&self, id: &str) -> Option<&MarketDef> {
        self.markets.iter().find(|m| m.id == id)
    }
    pub fn ending(&self, id: &str) -> Option<&EndingDef> {
        self.endings.iter().find(|m| m.id == id)
    }
    pub fn segment(&self, s: Segment) -> &SegmentDef {
        self.segments
            .iter()
            .find(|d| d.segment == s)
            .expect("chaque segment doit être défini dans le contenu")
    }
}
