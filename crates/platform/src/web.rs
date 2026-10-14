//! Adaptateur de stockage navigateur fondé sur `window.localStorage`.
//!
//! localStorage est synchrone et disponible partout ; une sauvegarde de campagne complète
//! pèse ~40-80 Ko de JSON, bien en dessous du quota de ~5 Mo par origine.
//! La navigation privée peut refuser l'écriture : les erreurs remontent jusqu'à l'interface.

use crate::storage::{Storage, StorageError};

pub struct LocalStorage {
    inner: web_sys::Storage,
}

// Le thread principal du navigateur est le seul thread sur wasm32-unknown-unknown.
unsafe impl Send for LocalStorage {}
unsafe impl Sync for LocalStorage {}

impl LocalStorage {
    pub fn new() -> Result<Self, StorageError> {
        let window = web_sys::window().ok_or_else(|| StorageError::Unavailable("no window".into()))?;
        let inner = window
            .local_storage()
            .map_err(|_| StorageError::Unavailable("localStorage refused".into()))?
            .ok_or_else(|| StorageError::Unavailable("localStorage missing".into()))?;
        Ok(Self { inner })
    }
}

const PREFIX: &str = "french-tech-tycoon:";

impl Storage for LocalStorage {
    fn get(&self, key: &str) -> Result<Option<String>, StorageError> {
        self.inner
            .get_item(&format!("{PREFIX}{key}"))
            .map_err(|_| StorageError::Io("getItem failed".into()))
    }
    fn set(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
        self.inner
            .set_item(&format!("{PREFIX}{key}"), value)
            .map_err(|_| StorageError::Io("setItem failed (quota or private mode)".into()))
    }
    fn remove(&mut self, key: &str) -> Result<(), StorageError> {
        self.inner
            .remove_item(&format!("{PREFIX}{key}"))
            .map_err(|_| StorageError::Io("removeItem failed".into()))
    }
}
