//! High-level manager orchestrating storage, filesystem service, stats, and rendering.

use std::io::{self, IsTerminal};
use std::path::PathBuf;

use crate::models::{DirectoryInfo, DirectoryStats};
use crate::render::render_entries_to;
use crate::services::DirectoryService;
use crate::stats::calculate_stats;
use crate::storage::{DirectoryStorage, PrefixLookup};

/// Embedded Bash/Zsh shell integration script.
pub const INIT_SH: &str = include_str!("shell/init.sh");

/// Embedded Fish shell integration script.
pub const INIT_FISH: &str = include_str!("shell/init.fish");

/// Main manager for temporary directory operations.
#[derive(Debug, Clone)]
pub struct TempitManager {
    pub storage: DirectoryStorage,
    pub service: DirectoryService,
}

impl TempitManager {
    pub fn new(storage_file: PathBuf, temp_base_dir: PathBuf) -> io::Result<Self> {
        Ok(Self {
            storage: DirectoryStorage::new(storage_file)?,
            service: DirectoryService::new(temp_base_dir),
        })
    }

    pub fn from_defaults() -> io::Result<Self> {
        Self::new(DirectoryStorage::default_path(), std::env::temp_dir())
    }

    pub fn init_script(shell: &str) -> Option<&'static str> {
        match shell {
            "bash" | "zsh" => Some(INIT_SH),
            "fish" => Some(INIT_FISH),
            _ => None,
        }
    }

    pub fn init_shell(&self, shell: &str) {
        if let Some(script) = Self::init_script(shell) {
            println!("{script}");
        } else {
            eprintln!("ERROR: Unsupported shell: {shell}");
        }
    }

    pub fn create(&self, prefix: &str) -> io::Result<PathBuf> {
        let dir_info = self.service.create_temp_directory(prefix)?;
        let path = dir_info.path.clone();
        self.storage.add_directory(dir_info)?;
        Ok(path)
    }

    pub fn remove(&self, number: i64) -> bool {
        if let Err(e) = self.storage.prune_stale() {
            eprintln!("ERROR: Error removing temporary directory: {e}");
            return false;
        }
        let Some(dir_path) = self.storage.get_path_by_number(number) else {
            return false;
        };
        if self.service.remove_directory(&dir_path) {
            if let Err(e) = self.storage.remove_directory(&dir_path) {
                eprintln!("ERROR: Error removing temporary directory: {e}");
                return false;
            }
            return true;
        }
        false
    }

    pub fn print_directories(&self) -> io::Result<()> {
        self.storage.prune_stale()?;
        let directories = self.storage.get_all_directories();
        let entries: Vec<(DirectoryInfo, DirectoryStats)> = directories
            .into_iter()
            .filter_map(|d| calculate_stats(&d).map(|s| (d, s)))
            .collect();
        let mut stdout = io::stdout().lock();
        let use_color = stdout.is_terminal();
        render_entries_to(&mut stdout, &entries, "Temporary Directories", use_color)
    }

    pub fn get_path_by_number(&self, number: i64) -> Option<PathBuf> {
        let _ = self.storage.prune_stale();
        self.storage.get_path_by_number(number)
    }

    pub fn get_most_recent_path(&self) -> Option<PathBuf> {
        let _ = self.storage.prune_stale();
        self.storage.get_most_recent_path()
    }

    pub fn get_path_by_prefix(&self, prefix: &str) -> PrefixLookup {
        let _ = self.storage.prune_stale();
        self.storage.get_path_by_prefix(prefix)
    }

    pub fn clean_all_directories(&self) -> io::Result<()> {
        self.storage.prune_stale()?;
        let directories = self.storage.get_all_directories();
        if directories.is_empty() {
            eprintln!("WARNING: No temporary directories found.");
            return Ok(());
        }
        for dir_info in directories {
            if self.service.remove_directory(&dir_info.path)
                && let Err(e) = self.storage.remove_directory(&dir_info.path)
            {
                eprintln!(
                    "ERROR: Error removing directory {}: {e}",
                    dir_info.path.display()
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn manager_lifecycle_and_prune() {
        let base = std::env::temp_dir().join(format!("tempit_mgr_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let storage_file = base.join("tempit_dirs.json");
        let work_base = base.join("dirs");
        fs::create_dir_all(&work_base).unwrap();

        let mgr = TempitManager::new(storage_file, work_base).unwrap();
        let p1 = mgr.create("alpha").unwrap();
        let p2 = mgr.create("beta").unwrap();
        assert!(p1.is_dir());
        assert!(p2.is_dir());

        fs::write(p1.join("hello.txt"), b"hello").unwrap();
        let all = mgr.storage.get_all_directories();
        assert_eq!(all.len(), 2);
        let stats = calculate_stats(&all[0]).unwrap();
        assert_eq!(stats.file_count, 1);
        assert_eq!(stats.size_bytes, 5);

        assert_eq!(mgr.get_path_by_number(1), Some(p1.clone()));
        assert_eq!(mgr.get_path_by_number(2), Some(p2.clone()));
        assert_eq!(mgr.get_path_by_number(99), None);

        assert!(mgr.remove(1));
        assert!(!p1.exists());
        assert_eq!(mgr.storage.get_all_directories().len(), 1);

        // Pure read does not prune stale entries
        fs::remove_dir_all(&p2).unwrap();
        assert_eq!(mgr.storage.get_all_directories().len(), 1);
        assert_eq!(mgr.storage.prune_stale().unwrap(), 1);
        assert!(mgr.storage.get_all_directories().is_empty());

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn get_most_recent_path_empty_returns_none() {
        let base =
            std::env::temp_dir().join(format!("tempit_mgr_recent_empty_{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let mgr = TempitManager::new(base.join("tempit_dirs.json"), base.join("dirs")).unwrap();

        assert_eq!(mgr.get_most_recent_path(), None);

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn get_most_recent_path_returns_latest_and_prunes_stale() {
        let base =
            std::env::temp_dir().join(format!("tempit_mgr_recent_prune_{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let mgr = TempitManager::new(base.join("tempit_dirs.json"), base.join("dirs")).unwrap();

        let p1 = mgr.create("first").unwrap();
        let p2 = mgr.create("second").unwrap();
        let p3 = mgr.create("third").unwrap();

        assert_eq!(mgr.get_most_recent_path(), Some(p3.clone()));

        // Remove newest directory on disk; get_most_recent_path should prune it and return p2
        fs::remove_dir_all(&p3).unwrap();
        assert_eq!(mgr.get_most_recent_path(), Some(p2.clone()));
        assert_eq!(mgr.storage.get_all_directories().len(), 2);

        // Remove remaining directories on disk; should prune all and return None
        fs::remove_dir_all(&p1).unwrap();
        fs::remove_dir_all(&p2).unwrap();
        assert_eq!(mgr.get_most_recent_path(), None);
        assert!(mgr.storage.get_all_directories().is_empty());

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn get_most_recent_path_uses_created_timestamp() {
        use chrono::NaiveDate;

        let base =
            std::env::temp_dir().join(format!("tempit_mgr_recent_ts_{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let dir_old = base.join("old");
        let dir_new = base.join("new");
        fs::create_dir_all(&dir_old).unwrap();
        fs::create_dir_all(&dir_new).unwrap();

        let mgr = TempitManager::new(base.join("tempit_dirs.json"), base.join("dirs")).unwrap();
        mgr.storage
            .add_directory(DirectoryInfo {
                path: dir_new.clone(),
                created: NaiveDate::from_ymd_opt(2026, 6, 1)
                    .unwrap()
                    .and_hms_opt(12, 0, 0)
                    .unwrap(),
                prefix: "new".to_string(),
            })
            .unwrap();
        mgr.storage
            .add_directory(DirectoryInfo {
                path: dir_old.clone(),
                created: NaiveDate::from_ymd_opt(2026, 1, 1)
                    .unwrap()
                    .and_hms_opt(12, 0, 0)
                    .unwrap(),
                prefix: "old".to_string(),
            })
            .unwrap();

        assert_eq!(mgr.get_most_recent_path(), Some(dir_new));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn get_path_by_prefix_exact_and_no_match() {
        let base =
            std::env::temp_dir().join(format!("tempit_mgr_prefix_exact_{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let mgr = TempitManager::new(base.join("tempit_dirs.json"), base.join("dirs")).unwrap();

        let p_api = mgr.create("api").unwrap();
        let p_web = mgr.create("web").unwrap();

        assert_eq!(mgr.get_path_by_prefix("api"), PrefixLookup::Matched(p_api));
        assert_eq!(mgr.get_path_by_prefix("web"), PrefixLookup::Matched(p_web));
        assert_eq!(mgr.get_path_by_prefix("unknown"), PrefixLookup::NotFound);
        assert_eq!(mgr.get_path_by_prefix(""), PrefixLookup::NotFound);

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn get_path_by_prefix_unique_prefix_and_exact_preference() {
        let base =
            std::env::temp_dir().join(format!("tempit_mgr_prefix_unique_{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let mgr = TempitManager::new(base.join("tempit_dirs.json"), base.join("dirs")).unwrap();

        let p_api = mgr.create("api").unwrap();
        let p_api_v2 = mgr.create("api-v2").unwrap();
        let p_frontend = mgr.create("frontend").unwrap();

        // Exact match is preferred over prefix match ("api" matches "api", not ambiguous with "api-v2")
        assert_eq!(mgr.get_path_by_prefix("api"), PrefixLookup::Matched(p_api));
        // Unique prefix match
        assert_eq!(
            mgr.get_path_by_prefix("front"),
            PrefixLookup::Matched(p_frontend)
        );
        assert_eq!(
            mgr.get_path_by_prefix("api-"),
            PrefixLookup::Matched(p_api_v2)
        );

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn get_path_by_prefix_ambiguous_and_prunes_stale() {
        let base = std::env::temp_dir().join(format!(
            "tempit_mgr_prefix_ambiguous_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let mgr = TempitManager::new(base.join("tempit_dirs.json"), base.join("dirs")).unwrap();

        let _p_other = mgr.create("other").unwrap();
        let p_api_v1 = mgr.create("api-v1").unwrap();
        let p_api_v2 = mgr.create("api-v2").unwrap();

        // Multiple prefix matches return Ambiguous with 1-based indices (2 and 3)
        match mgr.get_path_by_prefix("api") {
            PrefixLookup::Ambiguous(matches) => {
                assert_eq!(matches.len(), 2);
                assert_eq!(matches[0].index, 2);
                assert_eq!(matches[0].info.prefix, "api-v1");
                assert_eq!(matches[0].info.path, p_api_v1);
                assert_eq!(matches[1].index, 3);
                assert_eq!(matches[1].info.prefix, "api-v2");
                assert_eq!(matches[1].info.path, p_api_v2);
            }
            other => panic!("expected Ambiguous, got {other:?}"),
        }

        // Removing one directory on disk prunes it so "api" now uniquely matches api-v2
        fs::remove_dir_all(&p_api_v1).unwrap();
        assert_eq!(
            mgr.get_path_by_prefix("api"),
            PrefixLookup::Matched(p_api_v2.clone())
        );

        // Multiple exact matches return Ambiguous with only the exact matches
        let p_dup1 = mgr.create("dup").unwrap();
        let p_dup2 = mgr.create("dup").unwrap();
        let _p_dup_extra = mgr.create("dup-extra").unwrap();
        match mgr.get_path_by_prefix("dup") {
            PrefixLookup::Ambiguous(matches) => {
                assert_eq!(matches.len(), 2);
                assert_eq!(matches[0].index, 3);
                assert_eq!(matches[0].info.path, p_dup1);
                assert_eq!(matches[1].index, 4);
                assert_eq!(matches[1].info.path, p_dup2);
            }
            other => panic!("expected Ambiguous, got {other:?}"),
        }

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn init_script_returns_matching_script_or_none() {
        assert_eq!(TempitManager::init_script("bash"), Some(INIT_SH));
        assert_eq!(TempitManager::init_script("zsh"), Some(INIT_SH));
        assert_eq!(TempitManager::init_script("fish"), Some(INIT_FISH));
        assert_eq!(TempitManager::init_script("csh"), None);
        assert_eq!(TempitManager::init_script(""), None);
    }
}
