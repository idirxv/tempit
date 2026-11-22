# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

`tempit` is a standalone Rust CLI utility and shell helper for creating, tracking, and jumping to temporary directories.

## Commands

```bash
# Build
cargo build --release

# Run all unit + CLI integration tests
cargo test

# Format & lint
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

## Architecture

```
CLI (src/main.rs) → TempitManager (src/manager.rs)
                      ├── DirectoryStorage (src/storage.rs)  — atomic JSON persistence ($TEMPIT_STORAGE_FILE or <temp_dir>/tempit_dirs.json)
                      ├── DirectoryService (src/services.rs) — temp dir creation/removal only
                      ├── calculate_stats() (src/stats.rs)   — pure function: &DirectoryInfo → Option<DirectoryStats>
                      ├── humanize (src/humanize.rs)         — pure-Rust binary size & relative age formatters
                      └── render_entries_to() (src/render.rs)— comfy-table + owo-colors output
```

- **src/models.rs**: `DirectoryInfo` (`path`, `created`, `prefix`; JSON-serializable) and `DirectoryStats` (`size_bytes`, `human_size`, `file_count`, `dir_count`, `age`).
- **src/stats.rs**: Pure `calculate_stats(&dir_info)` function — no side effects, returns `None` for missing paths.
- **src/storage.rs**: `get_all_directories()` is a pure read; `prune_stale()` is the sole method that removes stale entries; writes are atomic (`tmp` + `rename`).
- **src/shell/init.sh**: Bash/Zsh integration embedded at compile time via `include_str!`, providing `tempc`, `tempg`, `templ`, `temprm`, `tempclean` aliases.
- **tests/cli_cases.rs**: 48 `trycmd` CLI regression cases in `tests/cases/`.

## Agent skills

### Issue tracker

Issues and specs are tracked as GitHub Issues on `idirxv/tempit` via the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

Default five-role vocabulary (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context layout (`GLOSSARY.md` and `docs/adr/` at the repo root). See `docs/agents/domain.md`.
