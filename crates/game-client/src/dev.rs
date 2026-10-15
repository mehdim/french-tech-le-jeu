//! Points d'automatisation pour les tests de fumée, les captures d'écran et la CI (natif uniquement).
//!
//! Variables d'environnement :
//!   FTT_START=new | auto:<jours>  saute le menu ; `auto` laisse l'autopilote
//!                                 jouer <jours> jours d'abord (grands bureaux)
//!   FTT_SHOTS=<sec>:<chemin>,...  prend des captures aux instants donnés
//!   FTT_EXIT=<sec>                quitte après ce nombre de secondes
//!   FTT_TAB=team|product|sales|funding|finance|office   ouvre un panneau
//!   FTT_SPEED=0|1|2|4             vitesse initiale de la simulation

use crate::sim::{DemoMode, Game, GameContent, SimClock};
use crate::states::AppState;
use crate::ui::{Tab, UiState};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::winit::{UpdateMode, WinitSettings};
use std::time::Duration;

#[derive(Resource, Default)]
struct DevConfig {
    start: Option<String>,
    shots: Vec<(f32, String)>,
    exit: Option<f32>,
    tab: Option<String>,
    speed: Option<u8>,
    started: bool,
}

pub struct DevPlugin;

impl Plugin for DevPlugin {
    fn build(&self, app: &mut App) {
        let env = |k: &str| {
            #[cfg(not(target_arch = "wasm32"))]
            {
                std::env::var(k).ok()
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = k;
                None::<String>
            }
        };
        let shots = env("FTT_SHOTS")
            .map(|s| {
                s.split(',')
                    .filter_map(|p| {
                        let (t, path) = p.split_once(':')?;
                        Some((t.parse().ok()?, path.to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default();
        app.insert_resource(DevConfig {
            start: env("FTT_START"),
            shots,
            exit: env("FTT_EXIT").and_then(|s| s.parse().ok()),
            tab: env("FTT_TAB"),
            speed: env("FTT_SPEED").and_then(|s| s.parse().ok()),
            started: false,
        })
        .add_systems(Update, (dev_start, dev_shots, frame_pacing));
    }
}

fn dev_start(
    mut cfg: ResMut<DevConfig>,
    mut game: ResMut<Game>,
    content: Res<GameContent>,
    mut demo: ResMut<DemoMode>,
    mut next: ResMut<NextState<AppState>>,
    mut clock: ResMut<SimClock>,
    mut ui: ResMut<UiState>,
) {
    if cfg.started {
        return;
    }
    cfg.started = true;
    let Some(start) = cfg.start.clone() else { return };
    let mut s = game_core::GameState::new(&content.0, 7);
    if let Some(days) = start.strip_prefix("auto:").and_then(|d| d.parse::<u32>().ok()) {
        while s.day < days && game_content::autopilot::step(&mut s, &content.0) {}
        s.campaign.intro_pending = false;
    }
    game.0 = s;
    demo.0 = false;
    clock.set_speed(cfg.speed.unwrap_or(1));
    if let Some(tab) = &cfg.tab {
        ui.right_open = true;
        ui.tab = match tab.as_str() {
            "product" => Tab::Product,
            "sales" => Tab::Sales,
            "funding" => Tab::Funding,
            "finance" => Tab::Finance,
            "office" => Tab::Office,
            "mentors" => Tab::Mentors,
            _ => Tab::Team,
        };
    }
    next.set(AppState::Playing);
}

fn dev_shots(mut commands: Commands, time: Res<Time>, mut cfg: ResMut<DevConfig>, mut exit: MessageWriter<AppExit>) {
    let t = time.elapsed_secs();
    let due: Vec<String> = cfg.shots.iter().filter(|(at, _)| t >= *at).map(|(_, p)| p.clone()).collect();
    if !due.is_empty() {
        cfg.shots.retain(|(at, _)| t < *at);
        for path in due {
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
        }
    }
    if cfg.exit.is_some_and(|e| t >= e) {
        exit.write(AppExit::Success);
    }
}

/// Mode « basse consommation » à 30 FPS pour les appareils plus lents (Paramètres → Performances).
fn frame_pacing(settings: Res<crate::saves::Settings>, mut winit: ResMut<WinitSettings>) {
    if !settings.is_changed() {
        return;
    }
    winit.focused_mode = if settings.low_power {
        UpdateMode::reactive(Duration::from_millis(33))
    } else {
        UpdateMode::Continuous
    };
}
