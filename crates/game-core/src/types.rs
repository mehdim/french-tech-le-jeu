//! Shared vocabulary: segments, roles, conditions, effects and modifiers.
//!
//! Content files (RON) are written in terms of these types, so they are the
//! contract between the simulation (`game-core`) and the data (`game-content`).

use serde::{Deserialize, Serialize};

/// Simulated days per accounting month.
pub const DAYS_PER_MONTH: u32 = 28;
/// Simulated days per week.
pub const DAYS_PER_WEEK: u32 = 7;

/// Customer segments. Each one has its own price, funnel and churn profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Segment {
    /// Self-serve SMBs, schools, freelancers.
    Smb,
    /// Media groups, universities, scale-ups.
    Mid,
    /// CAC 40 and public administrations.
    Enterprise,
}

impl Segment {
    pub const ALL: [Segment; 3] = [Segment::Smb, Segment::Mid, Segment::Enterprise];
    pub fn index(self) -> usize {
        match self {
            Segment::Smb => 0,
            Segment::Mid => 1,
            Segment::Enterprise => 2,
        }
    }
}

/// Employee roles. A role decides which work an employee is good at by
/// default; archetypes and traits refine it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Role {
    Founder,
    Backend,
    Frontend,
    AiResearcher,
    DevOps,
    Sales,
    Growth,
    ProductManager,
    Hr,
    Cfo,
    Intern,
    Consultant,
}

impl Role {
    pub const ALL: [Role; 12] = [
        Role::Founder,
        Role::Backend,
        Role::Frontend,
        Role::AiResearcher,
        Role::DevOps,
        Role::Sales,
        Role::Growth,
        Role::ProductManager,
        Role::Hr,
        Role::Cfo,
        Role::Intern,
        Role::Consultant,
    ];
    /// Roles that can be recruited (the founder is unique).
    pub const HIRABLE: [Role; 11] = [
        Role::Backend,
        Role::Frontend,
        Role::AiResearcher,
        Role::DevOps,
        Role::Sales,
        Role::Growth,
        Role::ProductManager,
        Role::Hr,
        Role::Cfo,
        Role::Intern,
        Role::Consultant,
    ];
    pub fn is_engineering(self) -> bool {
        matches!(
            self,
            Role::Founder
                | Role::Backend
                | Role::Frontend
                | Role::AiResearcher
                | Role::DevOps
                | Role::Intern
                | Role::Consultant
        )
    }
}

/// Temporary modifiers applied by events, investors and incidents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModKind {
    /// Multiplies lead generation.
    LeadMul,
    /// Multiplies lead-to-customer conversion.
    ConversionMul,
    /// Adds percentage points to monthly churn (all segments).
    ChurnAdd,
    /// Multiplies every employee's output.
    ProductivityMul,
    /// Adds to the morale target of the whole team.
    MoraleTarget,
    /// Multiplies infrastructure + inference costs.
    InfraCostMul,
    /// Multiplies sales capacity.
    SalesMul,
    /// Multiplies research output.
    ResearchMul,
    /// Multiplies production incident probability.
    IncidentRiskMul,
    /// Adds to competitor pressure (0..1 scale).
    CompetitionAdd,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Modifier {
    pub kind: ModKind,
    pub value: f64,
    pub remaining_days: u32,
    /// Short label shown in tooltips ("Influenceur LinkedIn", ...).
    #[serde(default)]
    pub source: String,
}

/// A predicate over the company state. Used by quests, event triggers,
/// choice requirements and investor demands.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Condition {
    Always,
    CashAtLeast(f64),
    CashBelow(f64),
    MrrAtLeast(f64),
    CustomersAtLeast(u32),
    PayingCustomersAtLeast(u32),
    SegmentCustomersAtLeast(Segment, u32),
    EmployeesAtLeast(u32),
    EmployeesBelow(u32),
    RoleAtLeast(Role, u32),
    /// Number of employees who joined during the current level.
    HiredInLevelAtLeast(u32),
    Shipped(String),
    NotShipped(String),
    /// Number of tasks with `meaningful: true` shipped during this level.
    ShippedInLevelAtLeast(u32),
    TaskActive(String),
    Flag(String),
    NotFlag(String),
    Unlocked(String),
    LevelAtLeast(u32),
    LevelAtMost(u32),
    QualityAtLeast(f64),
    QualityBelow(f64),
    /// Effective benchmark accuracy (model accuracy minus drift), in %.
    AccuracyAtLeast(f64),
    AccuracyBelow(f64),
    /// Effective false-positive rate, in %.
    FprAtMost(f64),
    FprAbove(f64),
    TechDebtAtLeast(f64),
    TechDebtBelow(f64),
    MoraleAtLeast(f64),
    MoraleBelow(f64),
    FatigueAtLeast(f64),
    ReputationAtLeast(f64),
    ReputationBelow(f64),
    /// Projected monthly churn across all paying customers, in %.
    ChurnBelow(f64),
    ChurnAtLeast(f64),
    SegmentChurnBelow(Segment, f64),
    RunwayAtLeast(f64),
    RunwayBelow(f64),
    GrossMarginAtLeast(f64),
    /// Non-dilutive + dilutive money secured during the current level
    /// (grants approved, loans, equity).
    FundingSecuredInLevelAtLeast(f64),
    /// A grant application was submitted during this level.
    ApplicationSubmittedInLevel,
    FounderEquityAtLeast(f64),
    /// An equity round of at least `min_amount` closed this level with
    /// dilution at most `max_dilution` percent.
    RoundClosedInLevel { min_amount: f64, max_dilution: f64 },
    EquityRaisedInLevelAtLeast(f64),
    /// MRR grew by at least `min_pct` % per month on average (compounded)
    /// over the last `months` closed months.
    MrrGrowthStreak { months: u32, min_pct: f64 },
    /// Inference cost per 1k analyses fell by at least this fraction
    /// (0.35 = 35 %) compared with the level start.
    InferenceCostReducedBy(f64),
    /// Paying customers are at least this % of the level-start count.
    CustomersRetainedPct(f64),
    MarketsAtLeast(u32),
    LanguagesAtLeast(u32),
    LtvCacAtLeast(f64),
    /// Monthly net burn (expenses - revenue) at most this amount.
    NetBurnAtMost(f64),
    ValuationAtLeast(f64),
    OfficeTierAtLeast(u32),
    DayInLevelAtLeast(u32),
    LeadsAtLeast(f64),
    Launched,
    All(Vec<Condition>),
    Any(Vec<Condition>),
    Not(Box<Condition>),
}

/// A state change. Used by event choices, task rewards and level starts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Effect {
    Cash(f64),
    Reputation(f64),
    MoraleAll(f64),
    FatigueAll(f64),
    /// Morale change for employees of one role only.
    MoraleRole(Role, f64),
    TechDebt(f64),
    /// Accuracy gain, scaled by the remaining gap to the theoretical ceiling.
    Accuracy(f64),
    /// False-positive rate change in percentage points (negative is good).
    Fpr(f64),
    /// Benchmark drift (positive degrades effective accuracy).
    Drift(f64),
    /// Multiplies the current drift (0.3 = remove 70 % of it).
    DriftMul(f64),
    /// Robustness to drift and adversarial editing (0..0.8).
    Robustness(f64),
    FeatureScore(f64),
    Leads(Segment, f64),
    /// Lose (negative) or gain a fraction of a segment's customers.
    CustomersPct(Segment, f64),
    AddCustomers(Segment, u32),
    CompetitorPressure(f64),
    /// Multiplies the inference cost per 1k analyses permanently.
    InferenceCostMul(f64),
    /// Adds to fixed monthly hosting costs.
    HostingMonthly(f64),
    /// Multiplies how much each customer uses the product.
    UsageMul(f64),
    /// Retention bonus for a segment (relative churn reduction, 0.08 = -8 %).
    Retention(Segment, f64),
    /// Conversion bonus for a segment (relative).
    ConversionBonus(Segment, f64),
    Languages(u32),
    SetFlag(String),
    ClearFlag(String),
    /// Extra paperwork on the active grant applications.
    AdminPoints(f64),
    /// Postpone pending grant/loan disbursements.
    DelayDisbursements(u32),
    Modifier { kind: ModKind, value: f64, days: u32 },
    /// Trigger a production incident right now.
    Incident,
    /// An employee with this role resigns (the most fatigued one).
    QuitRole(Role),
    /// Progress lost on active tasks (fraction 0..1).
    TaskSetback(f64),
    Unlock(String),
    /// Change the monthly salary of every employee by a fraction.
    SalaryPct(f64),
    /// Multiplies the founder's productivity permanently.
    FounderProductivity(f64),
    /// Monthly recurring income for N months (e.g. France Travail ARE/ARCE).
    Income { monthly: f64, months: u32, label: String },
    /// Queue a scripted event (story chains).
    TriggerEvent(String),
    /// The product goes from free beta to paid plans.
    Launch,
    /// Permanent reduction of production incident probability (0..0.8).
    Resilience(f64),
    /// Monthly rent change for the current office (reset when moving).
    RentMonthly(f64),
    /// Multiply every segment's price by (1 + f).
    PriceAll(f64),
}

/// Seniority bands affect salary and output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Seniority {
    Junior,
    Mid,
    Senior,
    Staff,
}

impl Seniority {
    pub fn output_mult(self) -> f64 {
        match self {
            Seniority::Junior => 0.8,
            Seniority::Mid => 1.0,
            Seniority::Senior => 1.2,
            Seniority::Staff => 1.4,
        }
    }
}
