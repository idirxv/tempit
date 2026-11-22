use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

struct TestEnv {
    root: PathBuf,
    storage_file: PathBuf,
    tmp_dir: PathBuf,
}

impl TestEnv {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("tempit_int_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let tmp_dir = root.join("tmp");
        fs::create_dir_all(&tmp_dir).unwrap();
        let storage_file = root.join("tempit_dirs.json");
        Self {
            root,
            storage_file,
            tmp_dir,
        }
    }

    fn tempit(&self) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_tempit"));
        cmd.env("TEMPIT_STORAGE_FILE", &self.storage_file)
            .env("TMPDIR", &self.tmp_dir);
        cmd
    }

    fn run_shell(&self, shell: &str, args: &[&str], script: &str, path_env: &str) -> Output {
        let mut child = Command::new(shell)
            .args(args)
            .current_dir(&self.root)
            .env("PATH", path_env)
            .env("TEMPIT_STORAGE_FILE", &self.storage_file)
            .env("TMPDIR", &self.tmp_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(script.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn bin_dir() -> PathBuf {
    Path::new(env!("CARGO_BIN_EXE_tempit"))
        .parent()
        .expect("tempit binary has parent dir")
        .to_path_buf()
}

#[test]
fn path_without_arg_returns_most_recent_and_prunes_stale() {
    let env = TestEnv::new("path_latest");

    let out1 = env.tempit().args(["create", "alpha"]).output().unwrap();
    assert!(out1.status.success());
    let dir1 = PathBuf::from(String::from_utf8(out1.stdout).unwrap().trim());

    let out2 = env.tempit().args(["create", "beta"]).output().unwrap();
    assert!(out2.status.success());
    let dir2 = PathBuf::from(String::from_utf8(out2.stdout).unwrap().trim());

    // `tempit path` with no arg returns the most recently created directory
    let latest = env.tempit().arg("path").output().unwrap();
    assert!(latest.status.success());
    assert_eq!(
        PathBuf::from(String::from_utf8(latest.stdout).unwrap().trim()),
        dir2
    );
    assert!(latest.stderr.is_empty());

    // Numeric index behavior remains unchanged
    let first = env.tempit().args(["path", "1"]).output().unwrap();
    assert!(first.status.success());
    assert_eq!(
        PathBuf::from(String::from_utf8(first.stdout).unwrap().trim()),
        dir1
    );

    // After removing the newest directory on disk, stale entry is pruned and previous is returned
    fs::remove_dir_all(&dir2).unwrap();
    let after_prune = env.tempit().arg("path").output().unwrap();
    assert!(after_prune.status.success());
    assert_eq!(
        PathBuf::from(String::from_utf8(after_prune.stdout).unwrap().trim()),
        dir1
    );

    // After removing all tracked directories, `tempit path` errors to stderr with empty stdout
    fs::remove_dir_all(&dir1).unwrap();
    let empty = env.tempit().arg("path").output().unwrap();
    assert!(!empty.status.success());
    assert!(empty.stdout.is_empty());
    let stderr = String::from_utf8(empty.stderr).unwrap();
    assert!(
        stderr.contains("ERROR: No temporary directories found."),
        "unexpected stderr: {stderr}"
    );
}

fn run_tempg_shell_test(shell: &str, shell_args: &[&str], enable_aliases: &str) {
    let env = TestEnv::new(&format!("shell_{shell}"));

    let out1 = env.tempit().args(["create", "first"]).output().unwrap();
    assert!(out1.status.success());
    let dir1 = fs::canonicalize(String::from_utf8(out1.stdout).unwrap().trim()).unwrap();

    let out2 = env.tempit().args(["create", "second"]).output().unwrap();
    assert!(out2.status.success());
    let dir2 = fs::canonicalize(String::from_utf8(out2.stdout).unwrap().trim()).unwrap();

    let path_env = format!(
        "{}:{}",
        bin_dir().display(),
        std::env::var("PATH").unwrap_or_default()
    );

    // `tempg` without args jumps to the most recently created directory; `tempg 1` jumps to #1
    let script = format!(
        r#"{enable_aliases}
eval "$(tempit init {shell})"
tempg
pwd -P
tempg 1
pwd -P
"#
    );
    let output = env.run_shell(shell, shell_args, &script, &path_env);

    assert!(
        output.status.success(),
        "{shell} failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2, "unexpected stdout: {stdout}");
    assert_eq!(PathBuf::from(lines[0]), dir2);
    assert_eq!(PathBuf::from(lines[1]), dir1);

    // `tempg` on empty storage returns non-zero, prints error to stderr, and does not change dir
    let empty_env = TestEnv::new(&format!("shell_empty_{shell}"));
    let empty_script = format!(
        r#"{enable_aliases}
eval "$(tempit init {shell})"
tempg
rc=$?
pwd -P
exit $rc
"#
    );
    let empty_out = empty_env.run_shell(shell, shell_args, &empty_script, &path_env);

    assert!(!empty_out.status.success());
    let stayed_dir = PathBuf::from(String::from_utf8(empty_out.stdout).unwrap().trim());
    assert_eq!(stayed_dir, fs::canonicalize(&empty_env.root).unwrap());
    let empty_stderr = String::from_utf8(empty_out.stderr).unwrap();
    assert!(
        empty_stderr.contains("ERROR: No temporary directories found."),
        "unexpected stderr: {empty_stderr}"
    );
}

#[test]
fn tempg_without_arg_works_in_bash() {
    run_tempg_shell_test(
        "bash",
        &["--norc", "--noprofile"],
        "shopt -s expand_aliases",
    );
}

#[test]
fn tempg_without_arg_works_in_zsh_if_installed() {
    if Command::new("zsh").arg("--version").output().is_err() {
        return;
    }
    run_tempg_shell_test("zsh", &["-f"], "setopt aliases");
}
