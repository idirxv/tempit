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
        let root = std::env::temp_dir().join(format!("tempit_comp_{name}_{}", std::process::id()));
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

    fn path_with_bin(&self) -> String {
        let bin_dir = Path::new(env!("CARGO_BIN_EXE_tempit"))
            .parent()
            .expect("tempit binary has parent dir");
        format!(
            "{}:{}",
            bin_dir.display(),
            std::env::var("PATH").unwrap_or_default()
        )
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn complete_subcommand_outputs_dynamic_candidates_and_handles_empty() {
    let env = TestEnv::new("cli_complete");

    // Empty storage: exit 0, empty stdout and stderr
    for kind in ["numbers", "targets"] {
        let out = env.tempit().args(["complete", kind]).output().unwrap();
        assert!(
            out.status.success(),
            "complete {kind} should succeed on empty"
        );
        assert!(out.stdout.is_empty(), "expected empty stdout for {kind}");
        assert!(out.stderr.is_empty(), "expected empty stderr for {kind}");
    }

    // Hidden from --help
    let help = env.tempit().arg("--help").output().unwrap();
    let help_str = String::from_utf8(help.stdout).unwrap();
    assert!(
        !help_str.contains("\n  complete"),
        "complete should be hidden from --help:\n{help_str}"
    );

    // Populate tracked directories
    assert!(
        env.tempit()
            .args(["create", "alpha"])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(
        env.tempit()
            .args(["create", "beta"])
            .output()
            .unwrap()
            .status
            .success()
    );

    let nums = env.tempit().args(["complete", "numbers"]).output().unwrap();
    assert!(nums.status.success());
    assert!(nums.stderr.is_empty());
    let num_vals: Vec<&str> = std::str::from_utf8(&nums.stdout)
        .unwrap()
        .lines()
        .map(|l| l.split('\t').next().unwrap())
        .collect();
    assert_eq!(num_vals, vec!["1", "2"]);

    let targets = env.tempit().args(["complete", "targets"]).output().unwrap();
    assert!(targets.status.success());
    assert!(targets.stderr.is_empty());
    let target_vals: Vec<&str> = std::str::from_utf8(&targets.stdout)
        .unwrap()
        .lines()
        .map(|l| l.split('\t').next().unwrap())
        .collect();
    assert_eq!(target_vals, vec!["1", "2", "alpha", "beta"]);
}

#[test]
fn bash_completion_registers_and_completes_subcommands_tempg_and_temprm() {
    let env = TestEnv::new("bash_exec");
    let path_env = env.path_with_bin();

    // Syntax check generated script with bash -n
    let init_out = env.tempit().args(["init", "bash"]).output().unwrap();
    assert!(init_out.status.success());
    let mut syntax = Command::new("bash")
        .arg("-n")
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    syntax
        .stdin
        .take()
        .unwrap()
        .write_all(&init_out.stdout)
        .unwrap();
    let syntax_out = syntax.wait_with_output().unwrap();
    assert!(
        syntax_out.status.success(),
        "bash -n failed: {}",
        String::from_utf8_lossy(&syntax_out.stderr)
    );

    let script = r#"
set -e
eval "$(tempit init bash)"

complete -p tempit >/dev/null
complete -p tempg >/dev/null
complete -p temprm >/dev/null

# Empty tracked dirs: no errors, empty COMPREPLY
COMP_WORDS=(tempg "")
COMP_CWORD=1
_tempit_go_complete
echo "EMPTY_GO:${#COMPREPLY[@]}"

COMP_WORDS=(temprm "")
COMP_CWORD=1
_tempit_rm_complete
echo "EMPTY_RM:${#COMPREPLY[@]}"

# Subcommand & flag completion
COMP_WORDS=(tempit "")
COMP_CWORD=1
_tempit_complete
echo "SUBCMDS:${COMPREPLY[*]}"

COMP_WORDS=(tempit init "")
COMP_CWORD=2
_tempit_complete
echo "INIT_SHELLS:${COMPREPLY[*]}"

# Create two tracked dirs and verify dynamic completion
tempit create alpha >/dev/null
tempit create beta >/dev/null

COMP_WORDS=(tempg "")
COMP_CWORD=1
_tempit_go_complete
echo "GO_ALL:${COMPREPLY[*]}"

COMP_WORDS=(tempg "al")
COMP_CWORD=1
_tempit_go_complete
echo "GO_FILTERED:${COMPREPLY[*]}"

COMP_WORDS=(temprm "")
COMP_CWORD=1
_tempit_rm_complete
echo "RM_ALL:${COMPREPLY[*]}"

COMP_WORDS=(tempit remove "")
COMP_CWORD=2
_tempit_complete
echo "TEMPIT_RM:${COMPREPLY[*]}"
"#;

    let out = Command::new("bash")
        .args(["--norc", "--noprofile", "-c", script])
        .env("PATH", &path_env)
        .env("TEMPIT_STORAGE_FILE", &env.storage_file)
        .env("TMPDIR", &env.tmp_dir)
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "bash completion test failed: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.stderr.is_empty(),
        "expected no stderr in bash completion, got: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("EMPTY_GO:0\n"), "stdout:\n{stdout}");
    assert!(stdout.contains("EMPTY_RM:0\n"), "stdout:\n{stdout}");
    assert!(
        stdout.contains("SUBCMDS:create init list remove clean-all help -v --version -h --help\n"),
        "stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("INIT_SHELLS:bash zsh fish -h --help\n"),
        "stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("GO_ALL:1 2 alpha beta\n"),
        "stdout:\n{stdout}"
    );
    assert!(stdout.contains("GO_FILTERED:alpha\n"), "stdout:\n{stdout}");
    assert!(stdout.contains("RM_ALL:1 2\n"), "stdout:\n{stdout}");
    assert!(stdout.contains("TEMPIT_RM:1 2\n"), "stdout:\n{stdout}");
}

#[test]
fn zsh_completion_registers_and_completes_if_zsh_installed() {
    if Command::new("zsh").arg("--version").output().is_err() {
        return;
    }

    let env = TestEnv::new("zsh_exec");
    let path_env = env.path_with_bin();
    let zcompdump = env.root.join("zcompdump");

    // Syntax check generated script with zsh -n
    let init_out = env.tempit().args(["init", "zsh"]).output().unwrap();
    assert!(init_out.status.success());
    let mut syntax = Command::new("zsh")
        .arg("-n")
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    syntax
        .stdin
        .take()
        .unwrap()
        .write_all(&init_out.stdout)
        .unwrap();
    let syntax_out = syntax.wait_with_output().unwrap();
    assert!(
        syntax_out.status.success(),
        "zsh -n failed: {}",
        String::from_utf8_lossy(&syntax_out.stderr)
    );

    // Sourcing without compinit must succeed with no errors
    let no_compinit = Command::new("zsh")
        .args(["-f", "-c", r#"eval "$(tempit init zsh)""#])
        .env("PATH", &path_env)
        .env("TEMPIT_STORAGE_FILE", &env.storage_file)
        .env("TMPDIR", &env.tmp_dir)
        .output()
        .unwrap();
    assert!(no_compinit.status.success());
    assert!(
        no_compinit.stderr.is_empty(),
        "unexpected stderr without compinit: {}",
        String::from_utf8_lossy(&no_compinit.stderr)
    );

    // Sourcing with compinit registers completions and dynamic candidate helpers work
    let script = format!(
        r#"
set -e
autoload -Uz compinit
compinit -u -d "{zcompdump}"
eval "$(tempit init zsh)"

echo "COMPDEF_TEMPIT:${{_comps[tempit]}}"
echo "COMPDEF_WRAPPER:${{_comps[_tempit]}}"
echo "COMPDEF_TEMPG:${{_comps[tempg]}}"
echo "COMPDEF_TEMPRM:${{_comps[temprm]}}"

# Stub _describe to capture candidates
_describe() {{
  shift 3
  local arr_name="$1"
  local -a vals
  eval "vals=(\"\${{(@)${{arr_name}}}}\")"
  echo "DESCRIBE:${{vals[*]}}"
}}

# Empty tracked dirs: should not call _describe or error
CURRENT=2
_tempit_go_complete
_tempit_rm_complete
echo "AFTER_EMPTY:ok"

tempit create alpha >/dev/null
tempit create beta >/dev/null

CURRENT=2
echo -n "GO:"
_tempit_go_complete
echo -n "RM:"
_tempit_rm_complete
"#,
        zcompdump = zcompdump.display()
    );

    let out = Command::new("zsh")
        .args(["-f", "-c", &script])
        .env("PATH", &path_env)
        .env("TEMPIT_STORAGE_FILE", &env.storage_file)
        .env("TMPDIR", &env.tmp_dir)
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "zsh completion test failed: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.stderr.is_empty(),
        "expected no stderr in zsh completion, got: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("COMPDEF_TEMPIT:_tempit_complete\n"),
        "stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("COMPDEF_WRAPPER:_tempit_wrapper_complete\n"),
        "stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("COMPDEF_TEMPG:_tempit_go_complete\n"),
        "stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("COMPDEF_TEMPRM:_tempit_rm_complete\n"),
        "stdout:\n{stdout}"
    );
    assert!(stdout.contains("AFTER_EMPTY:ok\n"), "stdout:\n{stdout}");
    assert!(
        stdout.contains("GO:DESCRIBE:") && stdout.contains("alpha:") && stdout.contains("beta:"),
        "expected tempg to suggest numbers and prefixes, stdout:\n{stdout}"
    );
    let rm_line = stdout
        .lines()
        .find(|l| l.starts_with("RM:DESCRIBE:"))
        .expect("RM:DESCRIBE line present");
    assert!(rm_line.contains("1:") && rm_line.contains("2:"));
    assert!(
        !rm_line.contains(" alpha:") && !rm_line.contains(" beta:"),
        "temprm should only suggest numbers, got: {rm_line}"
    );
}

#[test]
fn fish_completion_definitions_cover_subcommands_tempg_and_temprm() {
    let env = TestEnv::new("fish_comp");

    let out = env.tempit().args(["init", "fish"]).output().unwrap();
    assert!(out.status.success());
    assert!(out.stderr.is_empty());

    let script = String::from_utf8(out.stdout).unwrap();
    for expected in [
        "complete -c tempit -n __fish_use_subcommand -s v -l version",
        "complete -c tempit -n __fish_use_subcommand -s h -l help",
        "complete -c tempit -f -n __fish_use_subcommand -a create",
        "complete -c tempit -f -n __fish_use_subcommand -a init",
        "complete -c tempit -f -n __fish_use_subcommand -a list",
        "complete -c tempit -f -n __fish_use_subcommand -a remove",
        "complete -c tempit -f -n __fish_use_subcommand -a clean-all",
        "complete -c tempit -f -n \"__fish_seen_subcommand_from init",
        "complete -c tempit -f -n \"__fish_seen_subcommand_from remove",
        "complete -c tempg -f",
        "complete targets 2>/dev/null",
        "complete -c temprm -f",
        "complete numbers 2>/dev/null",
    ] {
        assert!(
            script.contains(expected),
            "expected fish init script to contain {expected:?}, got:\n{script}"
        );
    }

    // If fish is installed, also syntax-check and test live completion output
    if Command::new("fish").arg("--version").output().is_err() {
        return;
    }

    let path_env = env.path_with_bin();
    let fish_script = r#"
tempit init fish | source
set -l empty_go (complete -C 'tempg ')
set -l empty_rm (complete -C 'temprm ')
echo "EMPTY_GO:"(count $empty_go)
echo "EMPTY_RM:"(count $empty_rm)
tempit create alpha >/dev/null
tempit create beta >/dev/null
complete -C 'tempg ' | string replace -r '\t.*' '' | string join ' ' | read -l go_cands
complete -C 'temprm ' | string replace -r '\t.*' '' | string join ' ' | read -l rm_cands
echo "GO:$go_cands"
echo "RM:$rm_cands"
"#;

    let mut child = Command::new("fish")
        .arg("--no-config")
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
        .write_all(fish_script.as_bytes())
        .unwrap();
    let fish_out = child.wait_with_output().unwrap();
    assert!(
        fish_out.status.success(),
        "fish completion failed: {}",
        String::from_utf8_lossy(&fish_out.stderr)
    );
    assert!(fish_out.stderr.is_empty());
    let fish_stdout = String::from_utf8(fish_out.stdout).unwrap();
    assert!(
        fish_stdout.contains("EMPTY_GO:0\n"),
        "stdout:\n{fish_stdout}"
    );
    assert!(
        fish_stdout.contains("EMPTY_RM:0\n"),
        "stdout:\n{fish_stdout}"
    );
    assert!(
        fish_stdout.contains("GO:1 2 alpha beta\n"),
        "stdout:\n{fish_stdout}"
    );
    assert!(fish_stdout.contains("RM:1 2\n"), "stdout:\n{fish_stdout}");
}
