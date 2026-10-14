//! Pont entre Bevy et la simulation déterministe.
//!
//! La simulation avance par pas fixes d'un jour de jeu, indépendamment de la
//! fréquence d'affichage : un accumulateur convertit temps réel × vitesse en ticks entiers.

use crate::states::AppState;
use bevy::prelude::*;
use game_core::messages::Cue;
use game_core::{Content, GameState};

/// Contenu statique (niveaux, événements, chantiers...). Partagé, jamais modifié.
#[derive(Resource)]
pub struct GameContent(pub Content);

/// L'état de référence de l'entreprise. Dans le menu principal, c'est une entreprise
/// de démonstration pilotée par l'autopilote.
#[derive(Resource)]
pub struct Game(pub GameState);

/// Secondes réelles par jour simulé à 1×.
pub const SECONDS_PER_DAY: f32 = 1.6;
/// Jamais plus de ticks que cela en une seule image.
const MAX_TICKS_PER_FRAME: u32 = 4;

#[derive(Resource)]
pub struct SimClock {
    /// 0 = pause, 1, 2 ou 4.
    pub speed: u8,
    /// Vitesse à rétablir à la reprise.
    pub last_speed: u8,
    pub acc: f32,
    /// Fraction du jour en cours déjà écoulée (pour des animations fluides de l'interface).
    pub day_fraction: f32,
}

impl Default for SimClock {
    fn default() -> Self {
        Self { speed: 1, last_speed: 1, acc: 0.0, day_fraction: 0.0 }
    }
}

impl SimClock {
    pub fn toggle_pause(&mut self) {
        if self.speed == 0 {
            self.speed = self.last_speed.max(1);
        } else {
            self.last_speed = self.speed;
            self.speed = 0;
        }
    }
    pub fn set_speed(&mut self, s: u8) {
        self.speed = s;
        if s > 0 {
            self.last_speed = s;
        }
    }
}

/// Signaux émis par la simulation à cette image (sons, particules, notifications).
#[derive(Resource, Default)]
pub struct Cues(pub Vec<Cue>);

/// Indique si l'entreprise de démonstration est affichée à la place de celle du joueur.
#[derive(Resource, Default)]
pub struct DemoMode(pub bool);

pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        let content = game_content::load();
        let demo = demo_state(&content);
        app.insert_resource(Game(demo))
            .insert_resource(GameContent(content))
            .init_resource::<SimClock>()
            .init_resource::<Cues>()
            .insert_resource(DemoMode(true))
            .add_systems(PreUpdate, clear_cues)
            .add_systems(Update, tick_game.run_if(in_state(AppState::Playing)))
            .add_systems(Update, tick_demo.run_if(in_state(AppState::MainMenu)))
            .add_systems(Update, collect_cues.after(tick_game).after(tick_demo));
    }
}

/// Une entreprise de démonstration lancée depuis quelques mois, pour que le bureau de l'écran titre soit vivant.
pub fn demo_state(content: &Content) -> GameState {
    let mut s = GameState::new(content, 2026);
    while s.day < 120 && game_content::autopilot::step(&mut s, content) {}
    s
}

fn clear_cues(mut cues: ResMut<Cues>) {
    cues.0.clear();
}

fn tick_game(time: Res<Time>, mut clock: ResMut<SimClock>, mut game: ResMut<Game>, content: Res<GameContent>) {
    if clock.speed == 0 || !game.0.is_running() {
        return;
    }
    clock.acc += time.delta_secs() * clock.speed as f32 / SECONDS_PER_DAY;
    let mut n = 0;
    while clock.acc >= 1.0 && n < MAX_TICKS_PER_FRAME {
        clock.acc -= 1.0;
        if !game.0.tick(&content.0) {
            clock.acc = 0.0;
            break;
        }
        n += 1;
        if !game.0.is_running() {
            clock.acc = 0.0;
            break;
        }
    }
    if n == MAX_TICKS_PER_FRAME {
        clock.acc = clock.acc.min(1.0);
    }
    clock.day_fraction = clock.acc.fract();
}

/// Mode démo : l'autopilote joue une entreprise derrière l'écran titre.
fn tick_demo(time: Res<Time>, mut clock: ResMut<SimClock>, mut game: ResMut<Game>, content: Res<GameContent>, demo: Res<DemoMode>) {
    if !demo.0 {
        return;
    }
    clock.acc += time.delta_secs() / (SECONDS_PER_DAY * 0.6);
    while clock.acc >= 1.0 {
        clock.acc -= 1.0;
        if !game_content::autopilot::step(&mut game.0, &content.0) || game.0.day > 900 {
            game.0 = demo_state(&content.0);
        }
    }
    clock.day_fraction = clock.acc;
}

fn collect_cues(mut game: ResMut<Game>, mut cues: ResMut<Cues>) {
    if !game.0.cues.is_empty() {
        cues.0.append(&mut game.0.cues);
    }
}
