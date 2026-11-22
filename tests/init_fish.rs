use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

struct TestEnv {
    root: PathBuf,
    storage_file: PathBuf,
    tmp_dir: PathBuf,
}

impl TestEnv {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("tempit_init_{name}_{}", std::process::id()));
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
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn init_fish_outputs_valid_script_with_all_commands() {
    let env = TestEnv::new("fish_cli");

    let out = env.tempit().args(["init", "fish"]).output().unwrap();

    assert!(out.status.success());
    assert!(
        out.stderr.is_empty(),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let script = String::from_utf8(out.stdout).unwrap();
    for expected in [
        "function _tempit",
        "function tempc",
        "function tempg",
        "function templ",
        "function temprm",
        "function tempclean",
        "command \"$__TEMPIT_EXE\" create $argv",
        "command \"$__TEMPIT_EXE\" path $argv",
        "cd -- \"$__path\"",
    ] {
        assert!(
            script.contains(expected),
            "expected fish init script to contain {expected:?}, got:\n{script}"
        );
    }
}

#[test]
fn init_help_lists_fish_as_supported_shell() {
    let env = TestEnv::new("fish_help");

    let help = env.tempit().args(["init", "--help"]).output().unwrap();
    assert!(help.status.success());
    let help_stdout = String::from_utf8(help.stdout).unwrap();
    assert!(
        help_stdout.contains("fish"),
        "expected `tempit init --help` to mention fish, got:\n{help_stdout}"
    );

    let top_help = env.tempit().arg("--help").output().unwrap();
    assert!(top_help.status.success());
    let top_stdout = String::from_utf8(top_help.stdout).unwrap();
    assert!(
        top_stdout.contains("fish"),
        "expected `tempit --help` to mention fish, got:\n{top_stdout}"
    );
}

#[test]
fn init_unsupported_shell_prints_expected_error() {
    let env = TestEnv::new("unsup");

    let out = env.tempit().args(["init", "csh"]).output().unwrap();
    assert!(out.status.success());
    assert!(out.stdout.is_empty());
    assert_eq!(
        String::from_utf8(out.stderr).unwrap().trim(),
        "ERROR: Unsupported shell: csh"
    );
}

#[test]
fn fish_integration_works_if_installed() {
    if Command::new("fish").arg("--version").output().is_err() {
        return;
    }

    let env = TestEnv::new("fish_exec");

    let bin_dir = Path::new(env!("CARGO_BIN_EXE_tempit"))
        .parent()
        .expect("tempit binary has parent dir");
    let path_env = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let script = r#"
tempit init fish | source
tempc alpha
set -l d1 (pwd -P)
tempc beta
set -l d2 (pwd -P)
tempg 1
set -l g1 (pwd -P)
tempg
set -l glatest (pwd -P)
tempg alpha
set -l gprefix (pwd -P)
echo $d1
echo $d2
echo $g1
echo $glatest
echo $gprefix
templ >/dev/null
temprm 1
tempclean
"#;

    let mut child = Command::new("fish")
        .arg("--no-config")
        .current_dir(&env.root)
        .env("PATH", &path_env)
        .env("TEMPIT_STORAGE_FILE", &env.storage_file)
        .env("TMPDIR", &env.tmp_dir)
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
    let output = child.wait_with_output().unwrap();

    assert!(
        output.status.success(),
        "fish failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<PathBuf> = stdout.lines().map(PathBuf::from).collect();
    assert_eq!(lines.len(), 5, "unexpected stdout: {stdout}");
    assert_ne!(lines[0], lines[1]);
    assert_eq!(lines[2], lines[0], "tempg 1 should jump to first dir");
    assert_eq!(lines[3], lines[1], "tempg should jump to latest dir");
    assert_eq!(lines[4], lines[0], "tempg alpha should jump to alpha dir");
}
