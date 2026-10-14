//! Key/value storage used for save slots and settings.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    Unavailable(String),
    Io(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Unavailable(s) => write!(f, "stockage indisponible : {s}"),
            StorageError::Io(s) => write!(f, "erreur de stockage : {s}"),
        }
    }
}

/// Minimal persistent key/value store. Keys are short ASCII identifiers
/// (`save_slot_1`, `settings`); values are UTF-8 strings (JSON).
pub trait Storage: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<String>, StorageError>;
    fn set(&mut self, key: &str, value: &str) -> Result<(), StorageError>;
    fn remove(&mut self, key: &str) -> Result<(), StorageError>;
}

/// In-memory storage (tests, or when nothing else is available).
#[derive(Default)]
pub struct MemoryStorage {
    map: BTreeMap<String, String>,
}

impl Storage for MemoryStorage {
    fn get(&self, key: &str) -> Result<Option<String>, StorageError> {
        Ok(self.map.get(key).cloned())
    }
    fn set(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
        self.map.insert(key.to_string(), value.to_string());
        Ok(())
    }
    fn remove(&mut self, key: &str) -> Result<(), StorageError> {
        self.map.remove(key);
        Ok(())
    }
}

/// Desktop storage: one file per key in a directory.
#[cfg(not(target_arch = "wasm32"))]
pub struct FileStorage {
    dir: std::path::PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
impl FileStorage {
    pub fn new(dir: impl Into<std::path::PathBuf>) -> Self {
        Self { dir: dir.into() }
    }
    fn path(&self, key: &str) -> std::path::PathBuf {
        let safe: String = key.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' }).collect();
        self.dir.join(format!("{safe}.json"))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Storage for FileStorage {
    fn get(&self, key: &str) -> Result<Option<String>, StorageError> {
        match std::fs::read_to_string(self.path(key)) {
            Ok(s) => Ok(Some(s)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StorageError::Io(e.to_string())),
        }
    }
    fn set(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
        std::fs::create_dir_all(&self.dir).map_err(|e| StorageError::Io(e.to_string()))?;
        // Write to a temp file then rename, so a crash never leaves half a save.
        let tmp = self.path(&format!("{key}_tmp"));
        std::fs::write(&tmp, value).map_err(|e| StorageError::Io(e.to_string()))?;
        std::fs::rename(&tmp, self.path(key)).map_err(|e| StorageError::Io(e.to_string()))
    }
    fn remove(&mut self, key: &str) -> Result<(), StorageError> {
        match std::fs::remove_file(self.path(key)) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StorageError::Io(e.to_string())),
        }
    }
}

/// The best storage for the current platform.
pub fn default_storage() -> Box<dyn Storage> {
    #[cfg(target_arch = "wasm32")]
    {
        match crate::web::LocalStorage::new() {
            Ok(s) => Box::new(s),
            Err(_) => Box::new(MemoryStorage::default()),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let dir = std::env::var("FTT_SAVE_DIR").map(std::path::PathBuf::from).unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(|h| std::path::PathBuf::from(h).join(".local/share/french-tech-tycoon"))
                .unwrap_or_else(|_| std::path::PathBuf::from("saves"))
        });
        Box::new(FileStorage::new(dir))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_roundtrip() {
        let mut s = MemoryStorage::default();
        assert_eq!(s.get("a").unwrap(), None);
        s.set("a", "1").unwrap();
        assert_eq!(s.get("a").unwrap().as_deref(), Some("1"));
        s.remove("a").unwrap();
        assert_eq!(s.get("a").unwrap(), None);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn file_roundtrip() {
        let dir = std::env::temp_dir().join(format!("ftt_test_{}", std::process::id()));
        let mut s = FileStorage::new(&dir);
        s.set("save_slot_1", "{\"x\":1}").unwrap();
        assert_eq!(s.get("save_slot_1").unwrap().as_deref(), Some("{\"x\":1}"));
        s.remove("save_slot_1").unwrap();
        assert_eq!(s.get("save_slot_1").unwrap(), None);
        let _ = std::fs::remove_dir_all(dir);
    }
}
