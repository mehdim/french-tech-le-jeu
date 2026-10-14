//! État du moteur d'événements, aléatoires et scénarisés.

use crate::content::{Content, EventDef};
use crate::types::Effect;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PendingEvent {
    pub event_id: String,
    pub fired_day: u32,
    pub boss: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DelayedEffect {
    pub due_day: u32,
    pub chance: f64,
    pub effects: Vec<Effect>,
    pub text: String,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventRecord {
    pub day: u32,
    pub event_id: String,
    pub choice: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EventState {
    pub pending: Option<PendingEvent>,
    /// Événements scénarisés en attente que l'événement courant soit résolu.
    pub queue: Vec<PendingEvent>,
    pub last_fired: BTreeMap<String, u32>,
    pub fired_once: BTreeSet<String>,
    pub delayed: Vec<DelayedEffect>,
    pub last_event_day: u32,
    pub history: Vec<EventRecord>,
}

impl EventState {
    pub fn on_cooldown(&self, def: &EventDef, day: u32) -> bool {
        if def.once && self.fired_once.contains(&def.id) {
            return true;
        }
        match self.last_fired.get(&def.id) {
            Some(&d) => day < d + def.cooldown.max(1),
            None => false,
        }
    }

    pub fn pending_def<'a>(&self, content: &'a Content) -> Option<&'a EventDef> {
        self.pending.as_ref().and_then(|p| content.event(&p.event_id))
    }
}
