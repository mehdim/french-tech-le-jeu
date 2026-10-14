//! # game-core
//!
//! Deterministic, engine-agnostic simulation of VERIDIX, a French AI startup.
//! No Bevy, no browser APIs: this crate runs identically on web, desktop and
//! mobile, and in headless tests.
//!
//! * [`simulation::GameState`] is the single authoritative state.
//! * [`simulation::Command`] is the only way to mutate it from outside.
//! * [`content::Content`] holds static, data-driven definitions.

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
