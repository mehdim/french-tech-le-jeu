//! Definitions loaded from data files. The simulation only ever *reads*
//! these; everything mutable lives in [`crate::simulation::GameState`].

use crate::balancing::Balancing;
use crate::types::{Condition, Effect, Role, Segment, Seniority};
use serde::{Deserialize, Serialize};

/// One spoken line in a story scene.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Line {
    pub speaker: String,
    pub text: String,
}

/// Category of a development task (product tree branch).
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
    /// Technical debt added on completion (Standard approach).
    #[serde(default)]
    pub debt: f64,
    /// Probability (0..1) of a complication on completion.
    #[serde(default)]
    pub risk: f64,
    /// Can be run again after completion (maintenance tasks).
    #[serde(default)]
    pub repeatable: bool,
    /// Counts toward "ship N meaningful improvements".
    #[serde(default)]
    pub meaningful: bool,
    /// Short "expected benefit" text for the UI.
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

/// Personality traits with mechanical consequences (see `employees.rs`).
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
    /// Fixed contract length in days (consultants, interns).
    #[serde(default)]
    pub contract_days: Option<u32>,
    /// Daily rate billed instead of a salary (ESN consultants).
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
    /// Fee as a fraction of the hire's annual gross salary (recruiters).
    #[serde(default)]
    pub fee_pct: f64,
    pub delay: (u32, u32),
    pub candidates: (u32, u32),
    /// Archetype ids this channel can surface.
    pub archetypes: Vec<String>,
    #[serde(default)]
    pub requires: Vec<Condition>,
    /// Salary subsidy fraction for the first months (France Travail aid).
    #[serde(default)]
    pub subsidy: f64,
    /// Bonus paid on hire (referral bonus).
    #[serde(default)]
    pub hire_bonus: f64,
    /// Quality multiplier on rolled skills.
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
    /// Hosting/ops base cost added by the bigger infrastructure footprint.
    #[serde(default)]
    pub morale_bonus: f64,
    #[serde(default)]
    pub reputation_bonus: f64,
    /// Wall decorations displayed in this office.
    #[serde(default)]
    pub wall_decor: Vec<String>,
    /// Furniture added for free on move-in (layout x, y, item).
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
    /// Diminishing returns: only this many of the item count fully.
    #[serde(default = "max_eff")]
    pub max_effective: u32,
    #[serde(default)]
    pub requires_unlock: Option<String>,
    /// Sprite name in the client atlas.
    pub sprite: String,
}

fn max_eff() -> u32 {
    99
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProgramKind {
    Grant,
    /// Half grant, half repayable advance.
    Advance,
    Loan,
    /// Research tax credit computed from research payroll.
    TaxCredit,
    /// A status that lowers employer contributions for R&D staff.
    Status,
    /// Founder support (France Travail).
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
    /// (delay after approval in days, fraction of amount)
    pub tranches: Vec<(u32, f64)>,
    pub base_success: f64,
    pub cooldown_days: u32,
    /// Loans/advances: monthly repayment months after a grace period.
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
    /// Ticket size range in euros.
    pub ticket: (f64, f64),
    /// Valuation multiplier relative to the market model (<1 = tough).
    pub valuation_mult: f64,
    /// Metric this investor cares about (shown in UI).
    pub favourite_metric: String,
    /// Conditions for this investor to make an offer.
    #[serde(default)]
    pub requires: Vec<Condition>,
    /// Applied when the player accepts the offer.
    #[serde(default)]
    pub on_accept: Vec<Effect>,
    pub pitch_line: String,
    pub accept_line: String,
    pub reject_line: String,
    /// Investor demand displayed with the offer.
    #[serde(default)]
    pub demand: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoundDef {
    pub kind: RoundKind,
    pub name: String,
    pub requires_unlock: Option<String>,
    pub pitch_points: f64,
    /// Typical dilution range sought by investors in %.
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
    /// Text shown after choosing.
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
    /// Can only fire once per campaign.
    #[serde(default)]
    pub once: bool,
    /// Boss/scripted events are never picked randomly.
    #[serde(default)]
    pub scripted: bool,
    pub choices: Vec<EventChoice>,
    /// Effects applied as soon as the event fires (before choosing).
    #[serde(default)]
    pub immediate: Vec<Effect>,
    /// Sprite/illustration key for the client.
    #[serde(default)]
    pub art: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestKind {
    /// Latches once reached.
    Reach,
    /// Must hold at the moment the level is evaluated.
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
    /// Office tier the story moves you to when this level starts (if lower).
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

/// Default prices and the segment profile.
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
    /// Monthly analyses per customer.
    pub usage: f64,
    pub cost_per_lead: f64,
    pub requires_unlock: Option<String>,
    /// Shipped features required to sell properly in this segment.
    pub required_features: Vec<String>,
    pub tam_per_market: f64,
}

/// All static game content.
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
            .expect("every segment must be defined in content")
    }
}
