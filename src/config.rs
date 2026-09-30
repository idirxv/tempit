//! Everything tempit reads from its environment, resolved once at startup.

use std::env;
use std::path::{Path, PathBuf};

/// Where temporary directories live, where saved ones go, and who is asking.
#[derive(Debug, Clone)]
pub struct Config {
    /// Directory holding the tracked temporary directories.
    pub root: PathBuf,
    /// Default destination of `tempit save`, if one could be determined.
    pub save_dir: Option<PathBuf>,
    /// Working directory, which `.` refers to. `None` if it no longer exists.
    pub cwd: Option<PathBuf>,
    /// Name of the login shell (from `$SHELL`), for `tempit init` without argument.
    pub shell: Option<String>,
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
        let shell = env_path("SHELL").and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        });
        Self {
            root: absolute(root),
            save_dir: save_dir.map(absolute),
            cwd: env::current_dir().ok(),
            shell,
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
