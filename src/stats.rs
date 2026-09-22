//! Size, contents and age of a directory, for display.

use std::fs;
use std::path::Path;
use std::time::SystemTime;

use walkdir::WalkDir;

/// What `tempit list` shows about a directory.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DirStats {
    /// Total size of the regular files, in bytes.
    pub bytes: u64,
    pub files: u64,
    pub dirs: u64,
    /// Birth time where the filesystem records it, modification time otherwise.
    pub created: Option<SystemTime>,
}

impl DirStats {
    /// Walks `path` without following symlinks.
    ///
    /// Entries that cannot be read are skipped: the result is informational and should not
    /// make listing fail.
    pub fn collect(path: &Path) -> Self {
        let created = fs::metadata(path)
            .ok()
            .and_then(|meta| meta.created().or_else(|_| meta.modified()).ok());
        let mut stats = Self {
            created,
            ..Self::default()
        };
        for entry in WalkDir::new(path).min_depth(1).into_iter().flatten() {
            let file_type = entry.file_type();
            if file_type.is_dir() {
                stats.dirs += 1;
            } else if file_type.is_file() {
                stats.files += 1;
                stats.bytes += entry.metadata().map_or(0, |meta| meta.len());
            }
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::symlink;

    use super::*;

    #[test]
    fn counts_files_dirs_and_bytes_without_following_links() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("a/b")).unwrap();
        fs::write(root.join("one.txt"), "12345").unwrap();
        fs::write(root.join("a/b/two.txt"), "123").unwrap();
        symlink("/", root.join("link-to-root")).unwrap();

        let stats = DirStats::collect(root);

        assert_eq!((stats.files, stats.dirs, stats.bytes), (2, 2, 8));
        assert!(stats.created.is_some());
    }

    #[test]
    fn missing_directory_yields_empty_stats() {
        let stats = DirStats::collect(Path::new("/nonexistent/tempit/test"));
        assert_eq!(stats, DirStats::default());
    }
}
