//! Filesystem operations for creating and removing temporary directories.

use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::Local;

use crate::models::DirectoryInfo;

/// Creates and removes temporary directories on the filesystem.
#[derive(Debug, Clone)]
pub struct DirectoryService {
    pub temp_base_dir: PathBuf,
}

impl Default for DirectoryService {
    fn default() -> Self {
        Self {
            temp_base_dir: std::env::temp_dir(),
        }
    }
}

impl DirectoryService {
    pub fn new(temp_base_dir: PathBuf) -> Self {
        Self { temp_base_dir }
    }

    pub fn create_temp_directory(&self, prefix: &str) -> io::Result<DirectoryInfo> {
        let mut bytes = [0u8; 4];
        getrandom::fill(&mut bytes).map_err(|e| io::Error::other(e.to_string()))?;
        let mut hex = String::with_capacity(8);
        for b in bytes {
            let _ = write!(&mut hex, "{b:02x}");
        }
        let unique_name = format!("{prefix}_{hex}");
        let temp_dir = self.temp_base_dir.join(unique_name);
        if let Some(parent) = temp_dir.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::create_dir(&temp_dir)?;
        Ok(DirectoryInfo {
            path: temp_dir,
            created: Local::now().naive_local(),
            prefix: prefix.to_string(),
        })
    }

    pub fn remove_directory(&self, path: &Path) -> bool {
        if !path.exists() {
            eprintln!("WARNING: Directory does not exist: {}", path.display());
            return false;
        }
        match fs::remove_dir_all(path) {
            Ok(()) => true,
            Err(e) => {
                eprintln!("ERROR: Error removing directory {}: {e}", path.display());
                false
            }
        }
    }
}
