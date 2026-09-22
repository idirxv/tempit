//! End-to-end tests of the `tempit` binary.

mod common;

use std::fs;

use common::Sandbox;
use predicates::prelude::*;

fn line(path: impl AsRef<std::path::Path>) -> String {
    format!("{}\n", path.as_ref().display())
}

#[test]
fn create_numbers_directories_incrementally() {
    let sb = Sandbox::new();

    assert_eq!(sb.run(&["create"]), line(sb.root().join("1")));
    assert_eq!(
        sb.run(&["create", "bugfix"]),
        line(sb.root().join("2-bugfix"))
    );
    assert_eq!(sb.run(&["new"]), line(sb.root().join("3")));

    assert!(sb.root().join("2-bugfix").is_dir());
}

#[test]
fn create_rejects_invalid_labels() {
    let sb = Sandbox::new();
    sb.tempit()
        .args(["create", "my label"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("may only contain letters"));
    sb.tempit()
        .args(["create", "42"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("numbers refer to ids"));
}

#[test]
fn path_resolves_ids_labels_and_the_latest() {
    let sb = Sandbox::new();
    sb.run(&["create", "alpha"]);
    sb.run(&["create"]);

    assert_eq!(sb.run(&["path", "1"]), line(sb.root().join("1-alpha")));
    assert_eq!(sb.run(&["path", "alpha"]), line(sb.root().join("1-alpha")));
    assert_eq!(sb.run(&["path"]), line(sb.root().join("2")));
}

#[test]
fn path_reports_what_is_missing() {
    let sb = Sandbox::new();
    sb.tempit()
        .arg("path")
        .assert()
        .code(1)
        .stderr("tempit: there are no tracked directories\n");

    sb.run(&["create", "same"]);
    sb.run(&["create", "same"]);
    sb.tempit()
        .args(["path", "9"])
        .assert()
        .code(1)
        .stderr("tempit: no tracked directory matches id 9\n");
    sb.tempit()
        .args(["path", "same"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("(1-same, 2-same); use its id"));
}

#[test]
fn list_prints_a_plain_table_when_piped() {
    let sb = Sandbox::new();
    sb.run(&["create", "alpha"]);
    fs::write(sb.root().join("1-alpha/notes.txt"), "hello").unwrap();

    let out = sb.run(&["list"]);

    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2, "{out}");
    assert!(lines[0].starts_with("#  LABEL"));
    assert!(lines[1].starts_with("1  alpha  now"));
    assert!(lines[1].contains("5 B  1 file, 0 dirs"));
    assert!(!out.contains('\x1b'), "no colours when piped");
}

#[test]
fn list_of_nothing_explains_how_to_start() {
    let sb = Sandbox::new();
    sb.tempit()
        .arg("ls")
        .assert()
        .success()
        .stdout("")
        .stderr(predicate::str::contains("tempit create"));
}

#[test]
fn remove_keeps_other_ids_stable() {
    let sb = Sandbox::new();
    for _ in 0..3 {
        sb.run(&["create"]);
    }

    sb.tempit()
        .args(["remove", "2"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Removed"));

    assert_eq!(sb.run(&["__refs"]), "1\t\n3\t\n");
    assert_eq!(sb.run(&["create"]), line(sb.root().join("4")));
}

#[test]
fn remove_does_nothing_if_any_reference_is_wrong() {
    let sb = Sandbox::new();
    sb.run(&["create"]);
    sb.tempit().args(["rm", "1", "7"]).assert().code(1);
    assert!(sb.root().join("1").is_dir());
}

#[test]
fn remove_accepts_the_same_directory_twice() {
    let sb = Sandbox::new();
    sb.run(&["create", "x"]);
    sb.tempit().args(["rm", "1", "x"]).assert().success();
    assert_eq!(sb.run(&["__refs"]), "");
}

#[test]
fn clean_requires_confirmation() {
    let sb = Sandbox::new();
    sb.run(&["create"]);
    sb.run(&["create"]);

    // Tests have no terminal, so tempit must refuse rather than guess.
    sb.tempit()
        .arg("clean")
        .assert()
        .code(1)
        .stderr(predicate::str::contains("pass --yes"));
    assert_eq!(sb.run(&["__refs"]), "1\t\n2\t\n");

    sb.tempit()
        .args(["clean", "--yes"])
        .assert()
        .success()
        .stderr("Removed 2 directories.\n");
    assert_eq!(sb.run(&["__refs"]), "");

    // `clean-all` still works for users of the previous version.
    sb.tempit().args(["clean-all", "-y"]).assert().success();
}

#[test]
fn save_moves_to_the_save_directory_under_its_label() {
    let sb = Sandbox::new();
    sb.run(&["create", "keep"]);
    fs::write(sb.root().join("1-keep/notes.txt"), "important").unwrap();

    assert_eq!(sb.run(&["save", "keep"]), line(sb.save_dir().join("keep")));

    assert_eq!(
        fs::read_to_string(sb.save_dir().join("keep/notes.txt")).unwrap(),
        "important"
    );
    assert_eq!(
        sb.run(&["__refs"]),
        "",
        "a saved directory is no longer tracked"
    );
}

#[test]
fn save_names_unlabelled_directories_after_their_id() {
    let sb = Sandbox::new();
    sb.run(&["create"]);
    assert_eq!(sb.run(&["save", "1"]), line(sb.save_dir().join("tempit-1")));
}

#[test]
fn save_accepts_a_relative_destination() {
    let sb = Sandbox::new();
    sb.run(&["create"]);
    // The sandbox runs commands from its home directory.
    assert_eq!(
        sb.run(&["save", "1", "project"]),
        line(sb.home().join("project"))
    );
    assert!(sb.home().join("project").is_dir());
}

#[test]
fn save_never_overwrites() {
    let sb = Sandbox::new();
    sb.run(&["create"]);
    fs::write(sb.home().join("taken"), "").unwrap();

    sb.tempit()
        .args(["save", "1", "taken"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("already exists"));
    assert!(sb.root().join("1").is_dir());
}

#[test]
fn io_errors_include_the_path_and_the_cause() {
    let sb = Sandbox::new();
    let file = sb.home().join("not-a-dir");
    fs::write(&file, "").unwrap();
    sb.tempit()
        .arg("list")
        .env("TEMPIT_ROOT", &file)
        .assert()
        .code(1)
        .stderr(predicate::str::starts_with(format!(
            "tempit: cannot create {}: ",
            file.display()
        )));
}

#[test]
fn init_prints_the_integration_for_each_shell() {
    let sb = Sandbox::new();
    for shell in ["bash", "zsh"] {
        let script = sb.run(&["init", shell]);
        assert!(script.contains("tempc()"));
        assert!(script.contains("__tempit_refs()"));
    }
    assert!(sb.run(&["init", "bash"]).contains("complete -o default"));
    assert!(sb.run(&["init", "zsh"]).contains("compdef _tempit tempit"));

    sb.tempit().args(["init", "fish"]).assert().code(2);
    assert!(!sb.root().exists(), "init must not touch the filesystem");
}

#[test]
fn version_is_printed() {
    let sb = Sandbox::new();
    sb.tempit()
        .arg("--version")
        .assert()
        .success()
        .stdout(format!("tempit {}\n", env!("CARGO_PKG_VERSION")));
}
