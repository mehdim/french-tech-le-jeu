//! Structured log messages and presentation cues.
//!
//! The simulation never formats prose: it emits [`Msg`] values that the
//! content layer turns into French sentences (`game_content::dialogue`).
//! Text that already comes *from* content (event outcomes) uses `Text`.

use crate::types::{Role, Segment};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tone {
    Info,
    Good,
    Bad,
    Money,
    Story,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Msg {
    Text(String),
    Hired { name: String, role: Role },
    Resigned { name: String },
    Fired { name: String, severance: f64 },
    ContractEnded { name: String, consultant: bool },
    BurnOut { name: String, days: u32 },
    TaskStarted { task: String },
    TaskShipped { task: String },
    TaskComplication { task: String, debt: f64 },
    TaskCancelled { task: String, refund: f64 },
    Incident { cost: f64 },
    CustomersWeek { new: u32, churned: u32 },
    FirstCustomer { segment: Segment },
    MonthClosed { month: u32, net_burn: f64, mrr: f64 },
    CandidatesArrived { channel: String, count: u32 },
    NoCandidates { channel: String },
    ApplicationStarted { program: String },
    ApplicationSubmitted { program: String },
    ApplicationApproved { program: String, amount: f64 },
    ApplicationRejected { program: String },
    MoneyIn { label: String, amount: f64 },
    LoanRepaid { label: String },
    PitchStarted { round: String },
    OffersReceived { count: u32 },
    NoOffers,
    RoundClosed { investor: String, amount: f64, dilution: f64 },
    OffersExpired,
    LowRunway { months: f64 },
    NegativeCash { days_left: u32 },
    QuestDone { label: String },
    LevelComplete { level: u32 },
    OfficeMoved { name: String },
    FurniturePlaced { name: String },
    MarketOpened { name: String },
    PriceChanged { segment: Segment, price: f64 },
    IncomeEnded { label: String },
    Launched,
    DriftWarning { accuracy: f64 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogEntry {
    pub day: u32,
    pub tone: Tone,
    pub msg: Msg,
}

/// One-shot cues for sound and animation. Not saved.
#[derive(Clone, Debug, PartialEq)]
pub enum Cue {
    NewCustomers(u32),
    Churned(u32),
    Shipped(String),
    Incident,
    Funding(f64),
    Alert,
    Hired(u32),
    Left(u32),
    LevelComplete,
    EventPopup,
    Bankrupt,
    Victory,
    MonthClosed,
    Coffee,
}
