//! Helpers shared by the integration tests.

use std::path::{Path, PathBuf};

use assert_cmd::Command;

/// An isolated environment: its own root, save directory and home.
pub struct Sandbox {
    tmp: tempfile::TempDir,
}

impl Sandbox {
    pub fn new() -> Self {
        Self {
            tmp: tempfile::tempdir().expect("create sandbox"),
        }
    }

    pub fn home(&self) -> &Path {
        self.tmp.path()
    }

    pub fn root(&self) -> PathBuf {
        self.home().join("root")
    }

    pub fn save_dir(&self) -> PathBuf {
        self.home().join("saved")
    }

    /// Environment variables pointing tempit at this sandbox.
    pub fn vars(&self) -> [(&'static str, PathBuf); 3] {
        [
            ("TEMPIT_ROOT", self.root()),
            ("TEMPIT_SAVE_DIR", self.save_dir()),
            ("HOME", self.home().to_path_buf()),
        ]
    }

    /// A `tempit` command running inside this sandbox.
    pub fn tempit(&self) -> Command {
        let mut cmd = Command::new(tempit_bin());
        cmd.envs(self.vars()).current_dir(self.home());
        cmd
    }

    /// Runs `tempit` with `args`, asserts success and returns its stdout.
    pub fn run(&self, args: &[&str]) -> String {
        let output = self.tempit().args(args).assert().success();
        String::from_utf8(output.get_output().stdout.clone()).expect("utf-8 output")
    }
}

pub fn tempit_bin() -> &'static Path {
    assert_cmd::cargo::cargo_bin!("tempit")
}
