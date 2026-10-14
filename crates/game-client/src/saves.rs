//! Save slots, autosave and persisted settings.

use crate::sim::{DemoMode, Game, GameContent};
use crate::states::AppState;
use bevy::prelude::*;
use game_core::save::{SaveError, SaveMeta, from_save_string, read_meta, to_save_string};
use game_core::GameState;
use platform::Storage;
use serde::{Deserialize, Serialize};

pub const SLOTS: [&str; 4] = ["autosave", "slot_1", "slot_2", "slot_3"];

pub fn slot_label(slot: &str) -> &'static str {
    match slot {
        "autosave" => "Sauvegarde auto",
        "slot_1" => "Emplacement 1",
        "slot_2" => "Emplacement 2",
        _ => "Emplacement 3",
    }
}

#[derive(Resource)]
pub struct SaveStore(pub Box<dyn Storage>);

#[derive(Serialize, Deserialize, Clone, Resource, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub ui_scale: f32,
    /// Cap at 30 FPS for slower devices.
    pub low_power: bool,
    pub hints: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { music_volume: 0.5, sfx_volume: 0.7, ui_scale: 1.0, low_power: false, hints: true }
    }
}

/// Slot listing cache for the UI (refreshed after every write).
#[derive(Resource, Default)]
pub struct SlotList(pub Vec<(String, Result<SaveMeta, String>)>);

#[derive(Resource, Default)]
struct AutosaveTracker {
    level: u32,
    month: u32,
}

pub struct SavesPlugin;

impl Plugin for SavesPlugin {
    fn build(&self, app: &mut App) {
        let store = SaveStore(platform::default_storage());
        let settings = store
            .0
            .get("settings")
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str::<Settings>(&s).ok())
            .unwrap_or_default();
        app.insert_resource(store)
            .insert_resource(settings)
            .init_resource::<SlotList>()
            .init_resource::<AutosaveTracker>()
            .add_systems(Startup, refresh_slots_system)
            .add_systems(Update, (autosave.run_if(in_state(AppState::Playing)), persist_settings));
    }
}

pub fn refresh_slots(store: &SaveStore) -> SlotList {
    SlotList(
        SLOTS
            .iter()
            .filter_map(|slot| match store.0.get(slot) {
                Ok(Some(data)) => Some((slot.to_string(), read_meta(&data).map_err(|e| e.label()))),
                Ok(None) => None,
                Err(e) => Some((slot.to_string(), Err(e.to_string()))),
            })
            .collect(),
    )
}

fn refresh_slots_system(store: Res<SaveStore>, mut list: ResMut<SlotList>) {
    *list = refresh_slots(&store);
}

pub fn save_to(store: &mut SaveStore, slot: &str, game: &GameState, content: &GameContent) -> Result<(), String> {
    let label = content.0.level(game.campaign.level).map(|l| l.title.clone()).unwrap_or_default();
    let data = to_save_string(game, &label);
    store.0.set(slot, &data).map_err(|e| e.to_string())
}

pub fn load_from(store: &SaveStore, slot: &str, content: &GameContent) -> Result<GameState, String> {
    let data = store.0.get(slot).map_err(|e| e.to_string())?.ok_or_else(|| "Emplacement vide".to_string())?;
    from_save_string(&data, &content.0).map_err(|e: SaveError| e.label())
}

fn autosave(
    game: Res<Game>,
    demo: Res<DemoMode>,
    content: Res<GameContent>,
    mut store: ResMut<SaveStore>,
    mut tracker: ResMut<AutosaveTracker>,
    mut list: ResMut<SlotList>,
) {
    if demo.0 {
        return;
    }
    let level = game.0.campaign.level;
    let month = game.0.day / 28;
    if level != tracker.level || month != tracker.month {
        let first = tracker.level == 0;
        tracker.level = level;
        tracker.month = month;
        if !first || game.0.day > 0 {
            if let Err(e) = save_to(&mut store, "autosave", &game.0, &content) {
                warn!("autosave failed: {e}");
            }
            *list = refresh_slots(&store);
        }
    }
}

fn persist_settings(settings: Res<Settings>, mut store: ResMut<SaveStore>) {
    if settings.is_changed() && !settings.is_added()
        && let Ok(s) = serde_json::to_string(&*settings) {
            let _ = store.0.set("settings", &s);
        }
}
