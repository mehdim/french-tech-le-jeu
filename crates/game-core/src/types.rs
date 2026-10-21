//! Vocabulaire commun : segments, rôles, conditions, effets et modificateurs.
//!
//! Les fichiers de contenu (RON) sont écrits avec ces types : ils forment le
//! contrat entre la simulation (`game-core`) et les données (`game-content`).

use serde::{Deserialize, Serialize};

/// Jours simulés par mois comptable.
pub const DAYS_PER_MONTH: u32 = 28;
/// Jours simulés par semaine.
pub const DAYS_PER_WEEK: u32 = 7;

/// Segments de clientèle. Chacun a son prix, son tunnel et son profil de churn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Segment {
    /// PME en self-service, écoles, indépendants.
    Smb,
    /// Groupes de médias, universités, scale-ups.
    Mid,
    /// CAC 40 et administrations publiques.
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

/// Rôles des salariés. Un rôle détermine dans quel travail un salarié est bon
/// par défaut ; les profils et les traits l'affinent.
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
    /// Rôles recrutables (le fondateur est unique).
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

/// Modificateurs temporaires appliqués par les événements, les investisseurs et les incidents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModKind {
    /// Multiplie la génération de leads.
    LeadMul,
    /// Multiplie la conversion des leads en clients.
    ConversionMul,
    /// Ajoute des points de pourcentage au churn mensuel (tous segments).
    ChurnAdd,
    /// Multiplie la production de chaque salarié.
    ProductivityMul,
    /// S'ajoute à l'objectif de moral de toute l'équipe.
    MoraleTarget,
    /// Multiplie les coûts d'infrastructure et d'inférence.
    InfraCostMul,
    /// Multiplie la capacité commerciale.
    SalesMul,
    /// Multiplie la production de recherche.
    ResearchMul,
    /// Multiplie la probabilité d'incident de production.
    IncidentRiskMul,
    /// S'ajoute à la pression concurrentielle (échelle 0..1).
    CompetitionAdd,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Modifier {
    pub kind: ModKind,
    pub value: f64,
    pub remaining_days: u32,
    /// Court libellé affiché dans les infobulles (« Influenceur LinkedIn », ...).
    #[serde(default)]
    pub source: String,
}

/// Un prédicat sur l'état de l'entreprise. Utilisé par les objectifs, les déclencheurs
/// d'événements, les conditions des choix et les exigences des investisseurs.
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
    /// Nombre de salariés arrivés pendant le niveau en cours.
    HiredInLevelAtLeast(u32),
    Shipped(String),
    NotShipped(String),
    /// Nombre de chantiers `meaningful: true` livrés pendant ce niveau.
    ShippedInLevelAtLeast(u32),
    TaskActive(String),
    Flag(String),
    NotFlag(String),
    Unlocked(String),
    LevelAtLeast(u32),
    LevelAtMost(u32),
    QualityAtLeast(f64),
    QualityBelow(f64),
    /// Précision effective sur le benchmark (précision du modèle moins la dérive), en %.
    AccuracyAtLeast(f64),
    AccuracyBelow(f64),
    /// Taux effectif de faux positifs, en %.
    FprAtMost(f64),
    FprAbove(f64),
    TechDebtAtLeast(f64),
    TechDebtBelow(f64),
    MoraleAtLeast(f64),
    MoraleBelow(f64),
    FatigueAtLeast(f64),
    ReputationAtLeast(f64),
    ReputationBelow(f64),
    /// Churn mensuel projeté sur l'ensemble des clients payants, en %.
    ChurnBelow(f64),
    ChurnAtLeast(f64),
    SegmentChurnBelow(Segment, f64),
    RunwayAtLeast(f64),
    RunwayBelow(f64),
    GrossMarginAtLeast(f64),
    /// Argent non dilutif et dilutif obtenu pendant le niveau en cours
    /// (subventions accordées, prêts, levées).
    FundingSecuredInLevelAtLeast(f64),
    /// Un dossier de subvention a été déposé pendant ce niveau.
    ApplicationSubmittedInLevel,
    FounderEquityAtLeast(f64),
    /// Un tour de table d'au moins `min_amount` a été bouclé pendant ce niveau,
    /// avec une dilution d'au plus `max_dilution` pour cent.
    RoundClosedInLevel { min_amount: f64, max_dilution: f64 },
    EquityRaisedInLevelAtLeast(f64),
    /// Le MRR a progressé d'au moins `min_pct` % par mois en moyenne (composée)
    /// sur les `months` derniers mois clos.
    MrrGrowthStreak { months: u32, min_pct: f64 },
    /// Le coût d'inférence pour 1 000 analyses a baissé d'au moins cette fraction
    /// (0,35 = 35 %) par rapport au début du niveau.
    InferenceCostReducedBy(f64),
    /// Les clients payants représentent au moins ce % du nombre au début du niveau.
    CustomersRetainedPct(f64),
    MarketsAtLeast(u32),
    LanguagesAtLeast(u32),
    LtvCacAtLeast(f64),
    /// Burn net mensuel (dépenses - revenus) au plus égal à ce montant.
    NetBurnAtMost(f64),
    ValuationAtLeast(f64),
    OfficeTierAtLeast(u32),
    /// Le MRR est resté au moins à ce niveau sans interruption pendant `days` jours
    /// (relevés hebdomadaires).
    MrrHeldAtLeast { mrr: f64, days: u32 },
    /// Le marché (ou la filiale) est ouvert.
    MarketOpened(String),
    /// Un compteur d'histoire (recrues locales, accords d'État, contrats…) a atteint ce seuil.
    CounterAtLeast(String, u32),
    /// Ingénieur·es (backend, frontend, recherche, DevOps) affecté·es à plein temps
    /// au développement ou à l'infrastructure.
    EngineersFullTimeAtLeast(u32),
    /// L'énergie consommée pour 1 000 analyses a baissé d'au moins cette fraction
    /// par rapport au début du niveau.
    EnergyReducedBy(f64),
    DayInLevelAtLeast(u32),
    LeadsAtLeast(f64),
    Launched,
    All(Vec<Condition>),
    Any(Vec<Condition>),
    Not(Box<Condition>),
}

/// Un changement d'état. Utilisé par les choix d'événements, les récompenses de chantiers et les débuts de niveau.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Effect {
    Cash(f64),
    Reputation(f64),
    MoraleAll(f64),
    FatigueAll(f64),
    /// Changement de moral pour les salariés d'un seul rôle.
    MoraleRole(Role, f64),
    TechDebt(f64),
    /// Gain de précision, proportionnel à l'écart restant jusqu'au plafond théorique.
    Accuracy(f64),
    /// Variation du taux de faux positifs en points de pourcentage (négatif = mieux).
    Fpr(f64),
    /// Dérive du benchmark (positive = dégrade la précision effective).
    Drift(f64),
    /// Multiplie la dérive actuelle (0,3 = en retire 70 %).
    DriftMul(f64),
    /// Robustesse à la dérive et aux modifications adverses (0..0,8).
    Robustness(f64),
    FeatureScore(f64),
    Leads(Segment, f64),
    /// Perd (négatif) ou gagne une fraction des clients d'un segment.
    CustomersPct(Segment, f64),
    AddCustomers(Segment, u32),
    CompetitorPressure(f64),
    /// Multiplie durablement le coût d'inférence pour 1 000 analyses.
    InferenceCostMul(f64),
    /// S'ajoute aux coûts fixes mensuels d'hébergement.
    HostingMonthly(f64),
    /// Multiplie l'usage du produit par chaque client.
    UsageMul(f64),
    /// Bonus de rétention pour un segment (baisse relative du churn, 0,08 = -8 %).
    Retention(Segment, f64),
    /// Bonus de conversion pour un segment (relatif).
    ConversionBonus(Segment, f64),
    Languages(u32),
    SetFlag(String),
    ClearFlag(String),
    /// Paperasse supplémentaire sur les dossiers de subvention en cours.
    AdminPoints(f64),
    /// Repousse les versements de subventions/prêts en attente.
    DelayDisbursements(u32),
    Modifier { kind: ModKind, value: f64, days: u32 },
    /// Déclenche immédiatement un incident de production.
    Incident,
    /// Un salarié de ce rôle démissionne (le plus fatigué).
    QuitRole(Role),
    /// Avancement perdu sur les chantiers actifs (fraction 0..1).
    TaskSetback(f64),
    Unlock(String),
    /// Modifie le salaire mensuel de chaque salarié d'une fraction.
    SalaryPct(f64),
    /// Multiplie durablement la productivité du fondateur.
    FounderProductivity(f64),
    /// Revenu mensuel récurrent pendant N mois (ex. ARE/ARCE de France Travail).
    Income { monthly: f64, months: u32, label: String },
    /// Met un événement scénarisé en file d'attente (enchaînements de l'histoire).
    TriggerEvent(String),
    /// Le produit passe de la bêta gratuite aux offres payantes.
    Launch,
    /// Réduction permanente de la probabilité d'incident de production (0..0,8).
    Resilience(f64),
    /// Variation du loyer mensuel du bureau actuel (remise à zéro au déménagement).
    RentMonthly(f64),
    /// Multiplie le prix de chaque segment par (1 + f).
    PriceAll(f64),
    /// Choisit le mentor principal (sans effet si un mentor est déjà choisi).
    ChooseMentor(String),
    /// Ajoute (ou retire) une valeur à un compteur d'histoire.
    Counter(String, i32),
    /// Multiplie durablement l'énergie consommée pour 1 000 analyses.
    EnergyMul(f64),
    /// Refuse une offre de rachat : la défense commence (points de levée de fonds, puis combat).
    StartTakeover(String),
    /// Accepte le rachat par cette entreprise : la partie s'arrête.
    SellTo(String),
    /// Multiplie la trésorerie détenue en cryptomonnaie (krach, envolée, piratage…).
    CryptoMul(f64),
    /// Convertit toute la trésorerie crypto en euros, moyennant ces frais (0,03 = 3 %).
    CryptoSell(f64),
}

/// Les niveaux d'ancienneté influent sur le salaire et la production.
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
