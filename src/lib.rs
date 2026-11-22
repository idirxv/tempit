//! Tempit — Temporary Directory Manager.

pub mod humanize;
pub mod manager;
pub mod models;
pub mod render;
pub mod services;
pub mod stats;
pub mod storage;

pub use manager::{CompletionCandidate, CompletionKind, TempitManager};
pub use models::{DirectoryInfo, DirectoryStats};
pub use services::DirectoryService;
pub use stats::calculate_stats;
pub use storage::{DirectoryStorage, PrefixLookup, PrefixMatch};
