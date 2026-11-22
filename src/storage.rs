//! JSON-backed persistence for tracked directories with atomic writes.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::models::DirectoryInfo;

/// A candidate directory matched during prefix lookup, with its 1-based index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixMatch {
    pub index: usize,
    pub info: DirectoryInfo,
}

/// Result of looking up a tracked directory by prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefixLookup {
    /// Exactly one directory matched.
    Matched(PathBuf),
    /// Multiple directories matched; includes 1-based indices for disambiguation.
    Ambiguous(Vec<PrefixMatch>),
    /// No tracked directory matched the given prefix.
    NotFound,
}

/// Handles JSON persistence of tracked temporary directories.
#[derive(Debug, Clone)]
pub struct DirectoryStorage {
    pub storage_file: PathBuf,
}

impl DirectoryStorage {
    /// Default storage path: `$TEMPIT_STORAGE_FILE` or `<temp_dir>/tempit_dirs.json`.
    pub fn default_path() -> PathBuf {
        std::env::var_os("TEMPIT_STORAGE_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("tempit_dirs.json"))
    }

    /// Initialize storage, ensuring parent directory and JSON file exist.
    pub fn new(storage_file: PathBuf) -> io::Result<Self> {
        let storage = Self { storage_file };
        storage.ensure_storage_file()?;
        Ok(storage)
    }

    fn ensure_storage_file(&self) -> io::Result<()> {
        if let Some(parent) = self.storage_file.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }
        if !self.storage_file.exists() {
            self.write_directories(&[])?;
        }
        Ok(())
    }

    fn read_directories(&self) -> Vec<DirectoryInfo> {
        match fs::read_to_string(&self.storage_file) {
            Ok(text) => match serde_json::from_str::<Vec<DirectoryInfo>>(&text) {
                Ok(dirs) => dirs,
                Err(e) => {
                    eprintln!("WARNING: Error reading storage file: {e}");
                    Vec::new()
                }
            },
            Err(e) => {
                eprintln!("WARNING: Error reading storage file: {e}");
                Vec::new()
            }
        }
    }

    fn write_directories(&self, directories: &[DirectoryInfo]) -> io::Result<()> {
        let json = serde_json::to_string_pretty(directories)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let parent = self
            .storage_file
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let tmp_name = format!(
            ".{}.tmp.{}",
            self.storage_file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("tempit_dirs.json"),
            std::process::id()
        );
        let tmp_path = parent.join(tmp_name);
        fs::write(&tmp_path, json)?;
        if let Err(e) = fs::rename(&tmp_path, &self.storage_file) {
            let _ = fs::remove_file(&tmp_path);
            return Err(e);
        }
        Ok(())
    }

    pub fn add_directory(&self, dir_info: DirectoryInfo) -> io::Result<()> {
        let mut directories = self.read_directories();
        directories.push(dir_info);
        self.write_directories(&directories)
    }

    /// Pure read of all stored entries — no side effects.
    pub fn get_all_directories(&self) -> Vec<DirectoryInfo> {
        self.read_directories()
    }

    /// Remove entries whose directories no longer exist on disk.
    pub fn prune_stale(&self) -> io::Result<usize> {
        let all = self.read_directories();
        let total = all.len();
        let existing: Vec<DirectoryInfo> = all.into_iter().filter(|d| d.path.exists()).collect();
        let pruned = total - existing.len();
        if pruned > 0 {
            self.write_directories(&existing)?;
        }
        Ok(pruned)
    }

    /// 1-based lookup of a tracked directory path.
    pub fn get_path_by_number(&self, number: i64) -> Option<PathBuf> {
        let directories = self.get_all_directories();
        if number < 1 || (number as usize) > directories.len() {
            eprintln!("ERROR: Invalid directory number: {number}");
            return None;
        }
        Some(directories[(number - 1) as usize].path.clone())
    }

    /// Return the path of the most recently created tracked directory.
    pub fn get_most_recent_path(&self) -> Option<PathBuf> {
        self.get_all_directories()
            .into_iter()
            .max_by_key(|d| d.created)
            .map(|d| d.path)
    }

    /// Look up a tracked directory by prefix (exact match preferred, or unique prefix match).
    pub fn get_path_by_prefix(&self, query: &str) -> PrefixLookup {
        if query.is_empty() {
            return PrefixLookup::NotFound;
        }
        let mut exact = Vec::new();
        let mut prefixed = Vec::new();
        for (i, info) in self.get_all_directories().into_iter().enumerate() {
            if info.prefix == query {
                exact.push(PrefixMatch { index: i + 1, info });
            } else if info.prefix.starts_with(query) {
                prefixed.push(PrefixMatch { index: i + 1, info });
            }
        }
        let candidates = if !exact.is_empty() { exact } else { prefixed };
        match candidates.len() {
            0 => PrefixLookup::NotFound,
            1 => PrefixLookup::Matched(candidates.into_iter().next().unwrap().info.path),
            _ => PrefixLookup::Ambiguous(candidates),
        }
    }

    pub fn remove_directory(&self, path: &Path) -> io::Result<()> {
        let directories: Vec<DirectoryInfo> = self
            .read_directories()
            .into_iter()
            .filter(|d| d.path != path)
            .collect();
        self.write_directories(&directories)
    }
}
