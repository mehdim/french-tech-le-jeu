//! # game-core
//!
//! Simulation déterministe et indépendante du moteur de VERIDIX, une startup française d'IA.
//! Ni Bevy ni API navigateur : cette crate tourne à l'identique sur le web, le desktop et
//! le mobile, ainsi que dans les tests sans affichage.
//!
//! * [`simulation::GameState`] est l'unique état de référence.
//! * [`simulation::Command`] est le seul moyen de le modifier depuis l'extérieur.
//! * [`content::Content`] contient les définitions statiques, issues des données.

pub mod balancing;
pub mod conditions;
pub mod content;
pub mod customers;
pub mod economy;
pub mod effects;
pub mod employees;
pub mod events;
pub mod funding;
pub mod messages;
pub mod missions;
pub mod office;
pub mod product;
pub mod research;
pub mod save;
pub mod simulation;
pub mod types;

pub use content::Content;
pub use simulation::{Command, GameState};
