//! End-to-end tests of the shell integration: real shells evaluate `tempit init` and run the
//! `temp*` functions.

mod common;

use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

use common::{Sandbox, tempit_bin};

/// Runs `script` in `shell` after loading the integration, and returns its stdout.
fn run_in(shell: &str, sb: &Sandbox, script: &str) -> String {
    let bin_dir = tempit_bin().parent().expect("binary directory");
    let mut path = OsString::from(bin_dir);
    path.push(":");
    path.push(std::env::var_os("PATH").unwrap_or_default());

    let mut cmd = Command::new(shell);
    match shell {
        "bash" => cmd.args(["--norc", "--noprofile"]),
        "zsh" => cmd.arg("-f"),
        _ => unreachable!("unsupported shell {shell}"),
    };
    let output = cmd
        .arg("-c")
        .arg(format!("eval \"$(tempit init {shell})\"\nset -e\n{script}"))
        .envs(sb.vars())
        .env("PATH", path)
        .current_dir(sb.home())
        .output()
        .expect("run shell");
    assert!(
        output.status.success(),
        "{shell} failed: {}\nstdout: {}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    String::from_utf8(output.stdout).expect("utf-8 output")
}

fn lines(paths: &[&Path]) -> String {
    paths
        .iter()
        .map(|p| p.display().to_string() + "\n")
        .collect()
}

fn has_zsh() -> bool {
    Command::new("zsh").arg("-fc").arg("true").output().is_ok()
}

/// The behaviour every supported shell must share.
fn check_navigation(shell: &str) {
    let sb = Sandbox::new();
    let out = run_in(
        shell,
        &sb,
        "tempc alpha; pwd
         tempc; pwd
         tempg alpha; pwd
         tempg; pwd",
    );
    let (alpha, second) = (sb.root().join("1-alpha"), sb.root().join("2"));
    assert_eq!(out, lines(&[&alpha, &second, &alpha, &second]));
}

fn check_save_follows_the_shell(shell: &str) {
    // With an explicit reference, and without one: the directory you are in.
    for save in ["tempsave keep", "tempsave"] {
        let sb = Sandbox::new();
        let out = run_in(
            shell,
            &sb,
            &format!(
                "tempc keep; mkdir sub; cd sub
                 {save} 2>&1
                 pwd"
            ),
        );
        let saved = sb.save_dir().join("keep");
        let expected = format!(
            "Saved to {}\n{}",
            saved.display(),
            lines(&[&saved.join("sub")])
        );
        assert_eq!(out, expected, "{save}");
    }
}

fn check_remove_leaves_a_deleted_cwd(shell: &str) {
    let sb = Sandbox::new();
    let out = run_in(
        shell,
        &sb,
        "tempc gone; mkdir sub; cd sub; temprm . 2>&1; pwd",
    );
    let expected = format!(
        "Removed 1-gone\nThe current directory was deleted; moved to {home}\n{home}\n",
        home = sb.home().display()
    );
    assert_eq!(out, expected);
}

fn check_help_is_not_swallowed(shell: &str) {
    let sb = Sandbox::new();
    let out = run_in(shell, &sb, "tempc --help; pwd");
    assert!(out.contains("Usage: tempit create"), "{out}");
    assert!(out.ends_with(&lines(&[sb.home()])), "must not cd anywhere");
}

#[test]
fn bash_navigation() {
    check_navigation("bash");
}

#[test]
fn bash_save_follows_the_shell() {
    check_save_follows_the_shell("bash");
}

#[test]
fn bash_remove_leaves_a_deleted_cwd() {
    check_remove_leaves_a_deleted_cwd("bash");
}

#[test]
fn bash_help_is_not_swallowed() {
    check_help_is_not_swallowed("bash");
}

#[test]
fn bash_completion() {
    let sb = Sandbox::new();
    sb.run(&["create", "alpha"]);
    sb.run(&["create"]);
    let complete = |words: &str| {
        let script = format!(
            "COMP_WORDS=({words}); COMP_CWORD=$((${{#COMP_WORDS[@]}} - 1))
             __tempit_complete
             printf '%s\\n' \"${{COMPREPLY[@]}}\""
        );
        run_in("bash", &sb, &script)
    };
    let refs = "1\nalpha\n2\n";

    assert_eq!(complete("tempg ''"), refs);
    assert_eq!(complete("temprm 1 ''"), refs);
    assert_eq!(complete("tempit path al"), "alpha\n");
    assert_eq!(complete("tempit re"), "remove\n");
    assert_eq!(complete("tempit init z"), "zsh\n");
    assert_eq!(complete("tempclean -"), "--yes\n");
    // `save` completes a reference, then leaves the destination to file name completion.
    assert_eq!(complete("tempsave ''"), refs);
    assert_eq!(complete("tempsave 1 ''"), "\n");
}

#[test]
fn zsh_integration() {
    if !has_zsh() {
        eprintln!("zsh is not installed: skipping (CI installs it)");
        return;
    }
    check_navigation("zsh");
    check_save_follows_the_shell("zsh");
    check_remove_leaves_a_deleted_cwd("zsh");
    check_help_is_not_swallowed("zsh");

    // With the completion system loaded, every function gets its completer.
    let sb = Sandbox::new();
    let out = run_in(
        "zsh",
        &sb,
        "autoload -Uz compinit && compinit -u -D
         eval \"$(tempit init zsh)\"
         print -l $_comps[tempit] $_comps[tempg] $_comps[temprm] $_comps[tempsave]",
    );
    assert_eq!(
        out,
        "_tempit\n_tempit_tempg\n_tempit_temprm\n_tempit_tempsave\n"
    );
}
