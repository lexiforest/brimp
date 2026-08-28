use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use base64::Engine as _;

#[derive(Debug)]
pub struct PersistentStorage {
    root: PathBuf,
    quota_bytes: u64,
    lock: Mutex<()>,
}

impl PersistentStorage {
    pub fn new(root: impl Into<PathBuf>, quota_bytes: u64) -> Self {
        Self {
            root: root.into(),
            quota_bytes,
            lock: Mutex::new(()),
        }
    }

    pub fn list(&self, origin: &str, namespace: &str) -> Result<Vec<String>, String> {
        let _guard = self.lock.lock().map_err(|_| "storage lock poisoned")?;
        let directory = self.namespace_path(origin, namespace);
        let entries = match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.to_string()),
        };
        let mut keys = entries
            .filter_map(Result::ok)
            .filter_map(|entry| decode_component(&entry.file_name().to_string_lossy()))
            .collect::<Vec<_>>();
        keys.sort();
        Ok(keys)
    }

    pub fn get(&self, origin: &str, namespace: &str, key: &str) -> Result<Option<String>, String> {
        let _guard = self.lock.lock().map_err(|_| "storage lock poisoned")?;
        let path = self.entry_path(origin, namespace, key);
        match fs::read_to_string(path) {
            Ok(value) => Ok(Some(value)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn set(&self, origin: &str, namespace: &str, key: &str, value: &str) -> Result<(), String> {
        let _guard = self.lock.lock().map_err(|_| "storage lock poisoned")?;
        let path = self.entry_path(origin, namespace, key);
        let previous = fs::metadata(&path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        let usage = directory_size(&self.origin_path(origin))?;
        let next = usage
            .saturating_sub(previous)
            .saturating_add(value.len() as u64);
        if next > self.quota_bytes {
            return Err("QuotaExceededError".into());
        }
        let parent = path.parent().ok_or("invalid storage path")?;
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let temporary = path.with_extension("tmp");
        fs::write(&temporary, value).map_err(|error| error.to_string())?;
        fs::rename(temporary, path).map_err(|error| error.to_string())
    }

    pub fn delete(&self, origin: &str, namespace: &str, key: &str) -> Result<(), String> {
        let _guard = self.lock.lock().map_err(|_| "storage lock poisoned")?;
        let path = self.entry_path(origin, namespace, key);
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn clear(&self, origin: &str, namespace: &str) -> Result<(), String> {
        let _guard = self.lock.lock().map_err(|_| "storage lock poisoned")?;
        let path = self.namespace_path(origin, namespace);
        match fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn usage(&self, origin: &str) -> Result<u64, String> {
        let _guard = self.lock.lock().map_err(|_| "storage lock poisoned")?;
        directory_size(&self.origin_path(origin))
    }

    pub fn quota(&self) -> u64 {
        self.quota_bytes
    }

    fn entry_path(&self, origin: &str, namespace: &str, key: &str) -> PathBuf {
        self.namespace_path(origin, namespace)
            .join(encode_component(key))
    }

    fn namespace_path(&self, origin: &str, namespace: &str) -> PathBuf {
        self.origin_path(origin).join(encode_component(namespace))
    }

    fn origin_path(&self, origin: &str) -> PathBuf {
        self.root.join(encode_component(origin))
    }
}

fn encode_component(value: &str) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(value)
}

fn decode_component(value: &str) -> Option<String> {
    String::from_utf8(
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(value)
            .ok()?,
    )
    .ok()
}

fn directory_size(path: &Path) -> Result<u64, String> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error.to_string()),
    };
    let mut size = 0u64;
    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let metadata = entry.metadata().map_err(|error| error.to_string())?;
        size = size.saturating_add(if metadata.is_dir() {
            directory_size(&entry.path())?
        } else {
            metadata.len()
        });
    }
    Ok(size)
}
