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

    fn create_dir(&self, prefix: &str) -> PathBuf {
        let out = self.tempit().args(["create", prefix]).output().unwrap();
        assert!(
            out.status.success(),
            "create {prefix} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        PathBuf::from(String::from_utf8(out.stdout).unwrap().trim())
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
fn path_by_prefix_exact_unique_ambiguous_and_no_match() {
    let env = TestEnv::new("path_prefix");

    let dir_api = env.create_dir("api");
    let dir_api_v2 = env.create_dir("api-v2");
    let dir_frontend = env.create_dir("frontend");
    let dir_worker_a = env.create_dir("worker-a");
    let dir_worker_b = env.create_dir("worker-b");

    // Exact match is preferred over prefix match ("api" vs "api-v2")
    let exact = env.tempit().args(["path", "api"]).output().unwrap();
    assert!(exact.status.success());
    assert_eq!(
        PathBuf::from(String::from_utf8(exact.stdout).unwrap().trim()),
        dir_api
    );
    assert!(exact.stderr.is_empty());

    // Unique prefix match
    let unique = env.tempit().args(["path", "front"]).output().unwrap();
    assert!(unique.status.success());
    assert_eq!(
        PathBuf::from(String::from_utf8(unique.stdout).unwrap().trim()),
        dir_frontend
    );
    assert!(unique.stderr.is_empty());

    // Numeric 1-based lookup is preserved
    let by_num = env.tempit().args(["path", "2"]).output().unwrap();
    assert!(by_num.status.success());
    assert_eq!(
        PathBuf::from(String::from_utf8(by_num.stdout).unwrap().trim()),
        dir_api_v2
    );

    // Ambiguous prefix prints candidates with 1-based numbers to stderr and fails
    let ambig = env.tempit().args(["path", "worker"]).output().unwrap();
    assert!(!ambig.status.success());
    assert!(ambig.stdout.is_empty());
    let ambig_err = String::from_utf8(ambig.stderr).unwrap();
    assert!(
        ambig_err.contains("worker"),
        "expected query in stderr: {ambig_err}"
    );
    assert!(
        ambig_err.contains("4") && ambig_err.contains("worker-a"),
        "expected candidate #4 worker-a in stderr: {ambig_err}"
    );
    assert!(
        ambig_err.contains("5") && ambig_err.contains("worker-b"),
        "expected candidate #5 worker-b in stderr: {ambig_err}"
    );

    // No match prints error to stderr and fails
    let missing = env.tempit().args(["path", "nonexistent"]).output().unwrap();
    assert!(!missing.status.success());
    assert!(missing.stdout.is_empty());
    let missing_err = String::from_utf8(missing.stderr).unwrap();
    assert!(
        missing_err.contains("nonexistent"),
        "expected prefix in stderr: {missing_err}"
    );

    // Removing one ambiguous directory on disk prunes it and leaves a unique match
    fs::remove_dir_all(&dir_worker_a).unwrap();
    let resolved = env.tempit().args(["path", "worker"]).output().unwrap();
    assert!(resolved.status.success());
    assert_eq!(
        PathBuf::from(String::from_utf8(resolved.stdout).unwrap().trim()),
        dir_worker_b
    );
}

fn run_tempg_prefix_shell_test(shell: &str, shell_args: &[&str], enable_aliases: &str) {
    let env = TestEnv::new(&format!("shell_prefix_{shell}"));

    let dir_api = fs::canonicalize(env.create_dir("api")).unwrap();
    let _dir_api_v2 = fs::canonicalize(env.create_dir("api-v2")).unwrap();
    let dir_frontend = fs::canonicalize(env.create_dir("frontend")).unwrap();
    let _dir_worker_a = fs::canonicalize(env.create_dir("worker-a")).unwrap();
    let _dir_worker_b = fs::canonicalize(env.create_dir("worker-b")).unwrap();

    let path_env = format!(
        "{}:{}",
        bin_dir().display(),
        std::env::var("PATH").unwrap_or_default()
    );

    // Exact match, unique prefix match, and numeric lookup all change directory
    let script = format!(
        r#"{enable_aliases}
eval "$(tempit init {shell})"
tempg api
pwd -P
tempg front
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
    assert_eq!(lines.len(), 3, "unexpected stdout: {stdout}");
    assert_eq!(PathBuf::from(lines[0]), dir_api);
    assert_eq!(PathBuf::from(lines[1]), dir_frontend);
    assert_eq!(PathBuf::from(lines[2]), dir_api);

    let assert_tempg_fails = |arg: &str| -> String {
        let fail_script = format!(
            r#"{enable_aliases}
eval "$(tempit init {shell})"
tempg {arg}
rc=$?
pwd -P
exit $rc
"#
        );
        let out = env.run_shell(shell, shell_args, &fail_script, &path_env);
        assert!(!out.status.success());
        assert_eq!(
            PathBuf::from(String::from_utf8(out.stdout).unwrap().trim()),
            fs::canonicalize(&env.root).unwrap()
        );
        String::from_utf8(out.stderr).unwrap()
    };

    // Ambiguous prefix returns non-zero, prints candidates to stderr, and does not change dir
    let ambig_err = assert_tempg_fails("worker");
    assert!(
        ambig_err.contains("worker-a") && ambig_err.contains("worker-b"),
        "unexpected stderr: {ambig_err}"
    );

    // No match returns non-zero, prints error to stderr, and does not change dir
    let missing_err = assert_tempg_fails("nope");
    assert!(
        missing_err.contains("nope"),
        "unexpected stderr: {missing_err}"
    );
}

#[test]
fn tempg_by_prefix_works_in_bash() {
    run_tempg_prefix_shell_test(
        "bash",
        &["--norc", "--noprofile"],
        "shopt -s expand_aliases",
    );
}

#[test]
fn tempg_by_prefix_works_in_zsh_if_installed() {
    if Command::new("zsh").arg("--version").output().is_err() {
        return;
    }
    run_tempg_prefix_shell_test("zsh", &["-f"], "setopt aliases");
}
