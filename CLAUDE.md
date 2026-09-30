# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

tempit is a Rust CLI and shell helper for creating, tracking, jumping into and saving temporary
directories. Published on crates.io as `tempit` (v2+; v1.x was the `tempit-manager` Python package).

## Commands

```bash
cargo build
cargo test                                   # unit + tests/cli.rs + tests/shell.rs
cargo clippy --all-targets -- -D warnings    # pedantic lints are enabled in Cargo.toml
cargo fmt
shellcheck shell/common.sh && shellcheck --shell=bash shell/completion.bash
```

`tests/shell.rs` runs real bash (and zsh when installed) against the built binary.

## Architecture

```
main.rs   parse CLI (cli.rs), build Config (config.rs), dispatch, print errors
  ├── store.rs   Store: open (0700 root, owner check), list, create (flock'd, unique labels),
  │              resolve (+ "did you mean"), latest, containing (what `.` means)
  │              TrackedDir: remove, save_to
  ├── name.rs    Label / DirRef (id, label or `.`) / DirName: `<id>` or `<id>-<label>`
  ├── fsx.rs     move_dir: no-clobber rename, cross-device copy fallback (copy_tree)
  ├── stats.rs   DirStats::collect: size, counts, birth time (display only)
  ├── render.rs  aligned table + human_size / human_age
  └── shell.rs   embeds shell/common.sh + shell/completion.{bash,zsh}
```

- **The filesystem is the source of truth**: no index file. A directory is tracked iff it is in
  the root and its name parses as `DirName`. Ids are stable; next id = highest + 1. Labels
  are unique (checked under the lock).
- **Root**: `$TEMPIT_ROOT`, else `$TMPDIR/tempit-<uid>`. `save` defaults to `$TEMPIT_SAVE_DIR`
  or `~/tempit`. Environment (including the working directory for `.` and `$SHELL` for
  `init`) is read only in `Config::from_env`; everything else takes it as parameters.
- **Output contract**: stdout carries data only (paths, table, `__refs`, init script);
  messages and prompts go to stderr. Errors are one `Error` enum (thiserror), printed like
  clap's: `error: <msg>: <cause>` then `  tip: <what to do>` from `Error::tip()`, exit 1.
  Every error a user can hit should have a tip. `tempit init` on a terminal prints setup
  instructions instead of the script.
- **Shell integration**: `temp*` functions wrap the binary and never shadow `tempit`.
  Completion lists subcommands by hand; `shell::tests` fails if one is missing.
  Candidates come from the hidden `tempit __refs` command.
- `cargo package` verification shares `target/` and can leave a stale `target/debug/tempit`;
  run `cargo clean -p tempit` afterwards before trusting test results.

## Releasing

Bump `version` in `Cargo.toml`, push a matching `vX.Y.Z` tag. `.github/workflows/release.yml`
checks the tag, builds Linux (musl) and macOS binaries, creates the GitHub release and
publishes to crates.io via trusted publishing.


## Workflow Orchestration

### 1. Plan Mode Default

* Enter plan mode for ANY non-trivial task (3+ steps or architectural decisions)
* If something goes sideways, STOP and re-plan immediately – don't keep pushing
* Use plan mode for verification steps, not just building
* Write detailed specs upfront to reduce ambiguity

### 2. Subagent Strategy

* Use subagents liberally to keep main context window clean
* Offload research, exploration, and parallel analysis to subagents
* For complex problems, throw more compute at it via subagents
* One task per subagent for focused execution

### 3. Self-Improvement Loop

* After ANY correction from the user: update `tasks/lessons.md` with the pattern
* Write rules for yourself that prevent the same mistake
* Ruthlessly iterate on these lessons until mistake rate drops
* Review lessons at session start for relevant project

### 4. Verification Before Done

* Never mark a task complete without proving it works
* Diff behavior between main and your changes when relevant
* Ask yourself: "Would a staff engineer approve this?"
* Run tests, check logs, demonstrate correctness

### 5. Demand Elegance (Balanced)

* For non-trivial changes: pause and ask "is there a more elegant way?"
* If a fix feels hacky: "Knowing everything I know now, implement the elegant solution"
* Skip this for simple, obvious fixes – don't over-engineer
* Challenge your own work before presenting it

### 6. Autonomous Bug Fixing

* When given a bug report: just fix it. Don't ask for hand-holding
* Point at logs, errors, failing tests – then resolve them
* Zero context switching required from the user