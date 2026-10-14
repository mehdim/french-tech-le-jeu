//! Explicit top-level game states.

use crate::sim::{Game, SimClock};
use bevy::prelude::*;
use game_core::missions::Phase;

#[derive(States, Default, Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum AppState {
    /// Title screen; an autopilot company plays in the background.
    #[default]
    MainMenu,
    /// The player runs Veridix.
    Playing,
    /// Pause menu open (Esc). The simulation is frozen.
    Paused,
    /// A level was completed: outro + summary, then next level.
    MissionComplete,
    /// Bankruptcy.
    GameOver,
    /// The campaign is won; ending scene.
    Victory,
}

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_systems(Update, watch_phase.run_if(in_state(AppState::Playing)));
    }
}

/// Follow the simulation's campaign phase into the matching app state.
fn watch_phase(game: Res<Game>, mut next: ResMut<NextState<AppState>>, mut clock: ResMut<SimClock>) {
    match game.0.campaign.phase {
        Phase::Playing => {}
        Phase::LevelComplete => {
            clock.speed = 0;
            next.set(AppState::MissionComplete);
        }
        Phase::Bankrupt => {
            clock.speed = 0;
            next.set(AppState::GameOver);
        }
        Phase::Victory => {
            clock.speed = 0;
            next.set(AppState::Victory);
        }
    }
}
