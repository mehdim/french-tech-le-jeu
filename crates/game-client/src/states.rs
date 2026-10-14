//! États de jeu explicites de premier niveau.

use crate::sim::{Game, SimClock};
use bevy::prelude::*;
use game_core::missions::Phase;

#[derive(States, Default, Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum AppState {
    /// Écran titre ; une entreprise pilotée par l'autopilote joue en arrière-plan.
    #[default]
    MainMenu,
    /// Le joueur dirige Veridix.
    Playing,
    /// Menu pause ouvert (Échap). La simulation est figée.
    Paused,
    /// Un niveau est terminé : conclusion + bilan, puis niveau suivant.
    MissionComplete,
    /// Faillite.
    GameOver,
    /// La campagne est gagnée ; scène de fin.
    Victory,
}

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_systems(Update, watch_phase.run_if(in_state(AppState::Playing)));
    }
}

/// Fait suivre à l'état de l'application la phase de campagne de la simulation.
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
