//! Campaign progression: levels, quests and the level-start snapshot.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Playing,
    LevelComplete,
    Victory,
    Bankrupt,
}

/// Values captured when a level starts, for relative quests
/// ("reduce inference cost by 35 %", "keep 80 % of customers").
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LevelSnapshot {
    pub paying_customers: u32,
    pub inference_cost_per_k: f64,
    pub mrr: f64,
    pub employees: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LevelResult {
    pub level: u32,
    pub days: u32,
    pub cash_end: f64,
    pub mrr_end: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CampaignState {
    pub level: u32,
    pub level_start_day: u32,
    pub phase: Phase,
    pub quests_done: Vec<bool>,
    pub snapshot: LevelSnapshot,
    pub boss_fired: bool,
    pub unlocks: BTreeSet<String>,
    pub shipped_in_level: u32,
    pub hired_in_level: u32,
    pub ending: Option<String>,
    pub results: Vec<LevelResult>,
    /// The client shows the level intro scene while this is true.
    pub intro_pending: bool,
}

impl CampaignState {
    pub fn new() -> Self {
        Self {
            level: 0,
            level_start_day: 0,
            phase: Phase::Playing,
            quests_done: Vec::new(),
            snapshot: LevelSnapshot::default(),
            boss_fired: false,
            unlocks: BTreeSet::new(),
            shipped_in_level: 0,
            hired_in_level: 0,
            ending: None,
            results: Vec::new(),
            intro_pending: true,
        }
    }
    pub fn is_unlocked(&self, key: &str) -> bool {
        self.unlocks.contains(key)
    }
}

impl Default for CampaignState {
    fn default() -> Self {
        Self::new()
    }
}
