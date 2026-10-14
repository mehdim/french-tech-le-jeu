//! Format de sauvegarde versionné, avec somme de contrôle.
//!
//! Une sauvegarde est une petite enveloppe JSON autour du [`GameState`] sérialisé.
//! La somme de contrôle (FNV-1a 64) détecte les sauvegardes tronquées ou modifiées à la main ;
//! la version du schéma rejette les sauvegardes de builds incompatibles au lieu de
//! charger n'importe quoi. Seul l'état de référence de la simulation est stocké :
//! les entités de rendu sont reconstruites à partir de lui.

use crate::content::Content;
use crate::simulation::{GameState, SAVE_SCHEMA};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveMeta {
    pub level: u32,
    pub day: u32,
    pub cash: f64,
    pub mrr: f64,
    pub employees: u32,
    pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveFile {
    pub schema: u32,
    pub game_version: String,
    pub meta: SaveMeta,
    pub checksum: String,
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveError {
    Corrupt,
    Incompatible(u32),
    Parse(String),
}

impl SaveError {
    pub fn label(&self) -> String {
        match self {
            SaveError::Corrupt => "Sauvegarde corrompue (somme de contrôle invalide)".into(),
            SaveError::Incompatible(v) => format!("Sauvegarde d'une version incompatible (schéma {v})"),
            SaveError::Parse(e) => format!("Sauvegarde illisible : {e}"),
        }
    }
}

pub fn fnv1a(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in data {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

pub fn to_save_string(state: &GameState, label: &str) -> String {
    let body = serde_json::to_string(state).expect("l'état du jeu est toujours sérialisable");
    let file = SaveFile {
        schema: SAVE_SCHEMA,
        game_version: env!("CARGO_PKG_VERSION").to_string(),
        meta: SaveMeta {
            level: state.campaign.level,
            day: state.day,
            cash: state.finance.cash,
            mrr: state.metrics.mrr,
            employees: state.employees.len() as u32,
            label: label.to_string(),
        },
        checksum: format!("{:016x}", fnv1a(body.as_bytes())),
        state: body,
    };
    serde_json::to_string(&file).expect("l'enveloppe de sauvegarde est toujours sérialisable")
}

/// Lit seulement l'enveloppe (pour la liste des emplacements de sauvegarde).
pub fn read_meta(data: &str) -> Result<SaveMeta, SaveError> {
    let file: SaveFile = serde_json::from_str(data).map_err(|e| SaveError::Parse(e.to_string()))?;
    if file.schema != SAVE_SCHEMA {
        return Err(SaveError::Incompatible(file.schema));
    }
    if file.checksum != format!("{:016x}", fnv1a(file.state.as_bytes())) {
        return Err(SaveError::Corrupt);
    }
    Ok(file.meta)
}

pub fn from_save_string(data: &str, content: &Content) -> Result<GameState, SaveError> {
    let file: SaveFile = serde_json::from_str(data).map_err(|e| SaveError::Parse(e.to_string()))?;
    if file.schema != SAVE_SCHEMA {
        return Err(SaveError::Incompatible(file.schema));
    }
    if file.checksum != format!("{:016x}", fnv1a(file.state.as_bytes())) {
        return Err(SaveError::Corrupt);
    }
    let mut state: GameState =
        serde_json::from_str(&file.state).map_err(|e| SaveError::Parse(e.to_string()))?;
    state.after_load(content);
    Ok(state)
}
