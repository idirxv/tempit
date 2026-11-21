//! Data models for tracked temporary directories.

use std::path::PathBuf;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

fn default_prefix() -> String {
    "tempit".to_string()
}

/// Metadata for a tracked temporary directory (persisted as JSON).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectoryInfo {
    pub path: PathBuf,
    pub created: NaiveDateTime,
    #[serde(default = "default_prefix")]
    pub prefix: String,
}

/// Runtime statistics for a directory (computed on demand, not persisted).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectoryStats {
    pub size_bytes: u64,
    pub human_size: String,
    pub file_count: usize,
    pub dir_count: usize,
    pub age: String,
}
