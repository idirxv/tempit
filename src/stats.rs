//! Pure statistics calculation for tracked directories.

use std::fs;
use std::path::Path;

use chrono::Local;

use crate::humanize::{format_natural_time, format_size_binary};
use crate::models::{DirectoryInfo, DirectoryStats};

fn walk_stats(dir: &Path) -> (usize, usize, u64) {
    let mut file_count = 0usize;
    let mut dir_count = 0usize;
    let mut total_size = 0u64;
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        let Ok(entries) = fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(meta) = fs::symlink_metadata(&path) else {
                continue;
            };
            if meta.file_type().is_symlink() {
                continue;
            }
            if meta.is_file() {
                file_count += 1;
                total_size = total_size.saturating_add(meta.len());
            } else if meta.is_dir() {
                dir_count += 1;
                stack.push(path);
            }
        }
    }
    (file_count, dir_count, total_size)
}

/// Calculate runtime statistics for a directory. Returns `None` if the path does not exist.
pub fn calculate_stats(dir_info: &DirectoryInfo) -> Option<DirectoryStats> {
    if !dir_info.path.exists() {
        return None;
    }
    let (file_count, dir_count, size_bytes) = walk_stats(&dir_info.path);
    let now = Local::now().naive_local();
    let delta = now.signed_duration_since(dir_info.created);
    Some(DirectoryStats {
        size_bytes,
        human_size: format_size_binary(size_bytes),
        file_count,
        dir_count,
        age: format_natural_time(delta),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_dir_returns_none() {
        let info = DirectoryInfo {
            path: std::env::temp_dir().join("tempit_definitely_missing_dir_xyz"),
            created: Local::now().naive_local(),
            prefix: "missing".to_string(),
        };
        assert!(calculate_stats(&info).is_none());
    }

    #[test]
    fn counts_nested_files_and_dirs() {
        let root = std::env::temp_dir().join(format!("tempit_stats_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("sub")).unwrap();
        fs::write(root.join("a.txt"), b"hello").unwrap();
        fs::write(root.join("sub").join("b.txt"), b"world!").unwrap();

        let info = DirectoryInfo {
            path: root.clone(),
            created: Local::now().naive_local(),
            prefix: "test".to_string(),
        };
        let stats = calculate_stats(&info).unwrap();
        assert_eq!(stats.file_count, 2);
        assert_eq!(stats.dir_count, 1);
        assert_eq!(stats.size_bytes, 11);
        assert_eq!(stats.human_size, "11 Bytes");

        let _ = fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    struct TestDir(std::path::PathBuf);

    #[cfg(unix)]
    impl TestDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("{name}_{}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    #[cfg(unix)]
    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[cfg(unix)]
    fn stats_for(path: &Path) -> DirectoryStats {
        let info = DirectoryInfo {
            path: path.to_path_buf(),
            created: Local::now().naive_local(),
            prefix: "test".to_string(),
        };
        calculate_stats(&info).unwrap()
    }

    #[cfg(unix)]
    #[test]
    fn ignores_symlink_to_external_file_and_directory() {
        use std::os::unix::fs::symlink;

        let base = TestDir::new("tempit_stats_external_symlink");
        let tracked = base.path().join("tracked");
        let external = base.path().join("external");
        fs::create_dir_all(&tracked).unwrap();
        fs::create_dir_all(external.join("nested")).unwrap();

        fs::write(tracked.join("inside.txt"), b"hello").unwrap();
        fs::write(external.join("outside.txt"), b"external content").unwrap();
        fs::write(external.join("nested").join("deep.txt"), b"deep").unwrap();

        symlink(&external, tracked.join("link_dir")).unwrap();
        symlink(external.join("outside.txt"), tracked.join("link_file")).unwrap();

        let stats = stats_for(&tracked);
        assert_eq!(stats.file_count, 1);
        assert_eq!(stats.dir_count, 0);
        assert_eq!(stats.size_bytes, 5);
    }

    #[cfg(unix)]
    #[test]
    fn handles_cyclic_symlink_without_looping() {
        use std::os::unix::fs::symlink;

        let root = TestDir::new("tempit_stats_cyclic_symlink");
        let sub = root.path().join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("file.txt"), b"abc").unwrap();

        symlink(root.path(), sub.join("parent_loop")).unwrap();
        symlink(root.path(), root.path().join("self_loop")).unwrap();

        let stats = stats_for(root.path());
        assert_eq!(stats.file_count, 1);
        assert_eq!(stats.dir_count, 1);
        assert_eq!(stats.size_bytes, 3);
    }

    #[cfg(unix)]
    #[test]
    fn handles_broken_symlink_gracefully() {
        use std::os::unix::fs::symlink;

        let root = TestDir::new("tempit_stats_broken_symlink");
        fs::write(root.path().join("real.txt"), b"ok").unwrap();
        symlink(
            root.path().join("does_not_exist"),
            root.path().join("dangling"),
        )
        .unwrap();

        let stats = stats_for(root.path());
        assert_eq!(stats.file_count, 1);
        assert_eq!(stats.dir_count, 0);
        assert_eq!(stats.size_bytes, 2);
    }
}
