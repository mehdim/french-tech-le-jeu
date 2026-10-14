//! # platform
//!
//! Fines couches d'adaptation autour des services de la plateforme, pour que la logique
//! du jeu ne touche jamais directement une API du navigateur ou du système :
//!
//! * [`storage`] — persistance clé/valeur pour les sauvegardes et les paramètres
//!   (`localStorage` du navigateur sur le web, fichiers sur desktop ; un portage
//!   Android/iOS n'a besoin que d'une autre implémentation de `Storage`).
//! * [`clock`] — millisecondes de l'horloge murale pour tirer la graine des nouvelles parties.

pub mod storage;

#[cfg(target_arch = "wasm32")]
pub mod web;

pub mod clock {
    /// Millisecondes depuis l'époque Unix (utilisées uniquement pour la graine des nouvelles campagnes).
    pub fn now_millis() -> u64 {
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::Date::now() as u64
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0)
        }
    }
}

pub use storage::{Storage, StorageError, default_storage};
