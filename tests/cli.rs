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
        .stderr(predicate::str::contains(
            "labels may only contain letters, digits, '-', '_' and '.' (try 'my-label')",
        ));
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
    sb.tempit().arg("path").assert().code(1).stderr(
        "error: there are no temporary directories yet\n  \
             tip: create one with `tempit create [LABEL]`\n",
    );

    sb.run(&["create", "test"]);
    sb.tempit().args(["path", "9"]).assert().code(1).stderr(
        "error: no directory matches id 9\n  \
             tip: `tempit list` shows the ids and labels in use\n",
    );
    sb.tempit()
        .args(["path", "tst"])
        .assert()
        .code(1)
        .stderr("error: no directory matches label 'tst'\n  tip: did you mean 'test'?\n");

    // Duplicate labels can only come from directories created by hand.
    fs::create_dir(sb.root().join("7-dup")).unwrap();
    fs::create_dir(sb.root().join("8-dup")).unwrap();
    sb.tempit()
        .args(["path", "dup"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "label 'dup' matches several directories: 7-dup, 8-dup",
        ));
}

#[test]
fn labels_are_unique() {
    let sb = Sandbox::new();
    sb.run(&["create", "api"]);
    sb.tempit()
        .args(["create", "api"])
        .assert()
        .code(1)
        .stdout("")
        .stderr(
            "error: label 'api' is already used by 1-api\n  \
             tip: pick another label, or go there with `tempg api`\n",
        );
}

#[test]
fn dot_is_the_directory_you_are_in() {
    let sb = Sandbox::new();
    sb.run(&["create", "work"]);
    sb.run(&["create"]);
    let nested = sb.root().join("1-work/src/deep");
    fs::create_dir_all(&nested).unwrap();

    let path = sb
        .tempit()
        .args(["path", "."])
        .current_dir(&nested)
        .output();
    assert_eq!(
        String::from_utf8(path.unwrap().stdout).unwrap(),
        line(sb.root().join("1-work"))
    );

    sb.tempit()
        .args(["rm", "."])
        .current_dir(&nested)
        .assert()
        .success()
        .stderr("Removed 1-work\n");
    assert_eq!(sb.run(&["__refs"]), "2\t\n");
}

#[test]
fn dot_outside_any_directory_explains_itself() {
    let sb = Sandbox::new();
    sb.run(&["create"]);
    for args in [&["path", "."][..], &["save"]] {
        sb.tempit().args(args).assert().code(1).stderr(
            "error: the current directory is not inside a temporary directory\n  \
                 tip: `tempit list` shows the ids and labels in use\n",
        );
    }
}

#[test]
fn list_prints_a_plain_table_when_piped() {
    let sb = Sandbox::new();
    sb.run(&["create", "alpha"]);
    fs::write(sb.root().join("1-alpha/notes.txt"), "hello").unwrap();

    let expected = format!(
        "   #  LABEL  AGE  SIZE  CONTENTS\n   1  alpha  now   5 B  1 file\n\n\
         1 directory, 5 B in {}\n",
        sb.root().display()
    );
    assert_eq!(
        sb.run(&["list"]),
        expected,
        "plain, without colours, when piped"
    );
    assert_eq!(sb.run(&[]), expected, "`tempit` alone lists");
}

#[test]
fn list_marks_the_directory_you_are_in() {
    let sb = Sandbox::new();
    sb.run(&["create"]);
    sb.run(&["create", "here"]);
    let inside = sb.root().join("2-here/sub");
    fs::create_dir(&inside).unwrap();

    let output = sb
        .tempit()
        .arg("list")
        .current_dir(&inside)
        .output()
        .unwrap();
    let out = String::from_utf8(output.stdout).unwrap();

    assert!(out.contains("\n   1  -  "), "{out}");
    assert!(out.contains("\n▶  2  here  "), "{out}");
    assert!(out.ends_with("(▶ = current directory)\n"), "{out}");
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
        .stderr("Removed 2\n");

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
    sb.tempit().arg("clean").assert().code(1).stderr(
        "error: refusing to delete 2 directories (0 B) without confirmation\n  \
             tip: pass --yes to confirm\n",
    );
    assert_eq!(sb.run(&["__refs"]), "1\t\n2\t\n");

    sb.tempit()
        .args(["clean", "--yes"])
        .assert()
        .success()
        .stderr("Removed 2 directories (0 B).\n");
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
fn save_defaults_to_the_directory_you_are_in() {
    let sb = Sandbox::new();
    sb.run(&["create", "keep"]);
    let inside = sb.root().join("1-keep/sub");
    fs::create_dir(&inside).unwrap();

    sb.tempit()
        .arg("save")
        .current_dir(&inside)
        .assert()
        .success()
        .stdout(line(sb.save_dir().join("keep")));
    assert!(sb.save_dir().join("keep/sub").is_dir());
}

#[test]
fn save_rejects_a_path_given_as_reference() {
    let sb = Sandbox::new();
    sb.run(&["create"]);
    sb.tempit()
        .args(["save", "/tmp/project"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "expected an id, a label or '.', not a path",
        ));
}

#[test]
fn save_explains_a_missing_parent() {
    let sb = Sandbox::new();
    sb.run(&["create"]);
    sb.tempit()
        .args(["save", "1", "nope/project"])
        .assert()
        .code(1)
        .stderr(format!(
            "error: {} does not exist\n  tip: create it first, or save somewhere else\n",
            sb.home().join("nope").display()
        ));
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
        .stderr(format!(
            "error: cannot create {}: File exists\n",
            file.display()
        ));
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
fn init_detects_the_shell() {
    let sb = Sandbox::new();
    sb.tempit()
        .arg("init")
        .env("SHELL", "/usr/bin/zsh")
        .assert()
        .success()
        .stdout(predicate::str::contains("compdef _tempit tempit"));
    sb.tempit()
        .arg("init")
        .env("SHELL", "/usr/local/bin/fish")
        .assert()
        .code(1)
        .stderr(
            "error: unsupported shell 'fish'\n  \
             tip: tempit supports bash and zsh: `tempit init bash` or `tempit init zsh`\n",
        );
    sb.tempit()
        .arg("init")
        .env_remove("SHELL")
        .assert()
        .code(1)
        .stderr(predicate::str::contains("$SHELL is not set"));
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
