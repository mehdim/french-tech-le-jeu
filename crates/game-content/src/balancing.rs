//! Chargement des paramètres d'équilibrage (`assets/data/balancing.ron`).

use crate::ContentError;
use game_core::balancing::Balancing;

pub fn parse(src: &str) -> Result<Balancing, ContentError> {
    ron::from_str(src).map_err(|e| ContentError(format!("balancing.ron: {e}")))
}
