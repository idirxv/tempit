//! Locations tempit works with, resolved once from the environment.

use std::env;
use std::path::{Path, PathBuf};

/// Where temporary directories live and where saved ones go by default.
#[derive(Debug, Clone)]
pub struct Config {
    /// Directory holding the tracked temporary directories.
    pub root: PathBuf,
    /// Default destination of `tempit save`, if one could be determined.
    pub save_dir: Option<PathBuf>,
}

impl Config {
    /// Reads `TEMPIT_ROOT` and `TEMPIT_SAVE_DIR`, falling back to `$TMPDIR/tempit-<uid>`
    /// and `~/tempit`.
    pub fn from_env() -> Self {
        let root = env_path("TEMPIT_ROOT").unwrap_or_else(|| {
            let uid = rustix::process::getuid().as_raw();
            env::temp_dir().join(format!("tempit-{uid}"))
        });
        let save_dir =
            env_path("TEMPIT_SAVE_DIR").or_else(|| env::home_dir().map(|home| home.join("tempit")));
        Self {
            root: absolute(root),
            save_dir: save_dir.map(absolute),
        }
    }
}

fn env_path(key: &str) -> Option<PathBuf> {
    env::var_os(key)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// Makes `path` absolute (without resolving symlinks) so that printed paths stay valid
/// after the shell changes directory.
pub fn absolute(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}
