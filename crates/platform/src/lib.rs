//! # platform
//!
//! Thin adapters around platform services so the game logic never touches a
//! browser or OS API directly:
//!
//! * [`storage`] — key/value persistence for saves and settings
//!   (browser `localStorage` on the web, files on desktop; an Android/iOS
//!   port only needs another `Storage` implementation).
//! * [`clock`] — wall-clock milliseconds for seeding new games.

pub mod storage;

#[cfg(target_arch = "wasm32")]
pub mod web;

pub mod clock {
    /// Milliseconds since the Unix epoch (used only to seed new campaigns).
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
