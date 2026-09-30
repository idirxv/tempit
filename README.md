# Tempit — Temporary Directory Manager

[![CI](https://github.com/idirxv/tempit/actions/workflows/ci.yml/badge.svg)](https://github.com/idirxv/tempit/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/tempit.svg)](https://crates.io/crates/tempit)
[![Codacy Badge](https://app.codacy.com/project/badge/Grade/355fe09860a44384a5efe8580fbfc20a)](https://app.codacy.com/gh/idirxv/tempit/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Create throwaway working directories, jump between them, and keep the ones that turned out
to matter, without losing track of anything in `/tmp`.

```console
$ tempc bugfix     # create /tmp/tempit-1000/1-bugfix and cd into it
$ tempc            # create /tmp/tempit-1000/2 and cd into it
$ tempg bugfix     # back to bugfix (or: tempg 1; plain tempg = the latest)
$ templ            # or just: tempit
   #  LABEL   AGE     SIZE  CONTENTS
▶  1  bugfix  12m  1.2 MiB  4 files, 1 dir
   2  -       now      0 B  empty

2 directories, 1.2 MiB in /tmp/tempit-1000
(▶ = current directory)
$ tempsave         # keep the one you are in: moved to ~/tempit/bugfix, your shell follows
$ temprm 2         # delete one (temprm . deletes the one you are in)
$ tempclean        # delete them all: shows what will go, then asks
```

## Install

With Cargo:

```bash
cargo install tempit
```

Or download a prebuilt binary (Linux x86_64/aarch64, static; macOS Intel/Apple Silicon) from
the [releases page](https://github.com/idirxv/tempit/releases) and put it on your `PATH`.

Then enable the shell integration, which provides the `temp*` functions and tab completion.
Run `tempit init` to see what to add for your shell, or add it directly:

```bash
# ~/.bashrc
eval "$(tempit init bash)"

# ~/.zshrc (after compinit)
eval "$(tempit init zsh)"
```

## Usage

| Shell function | Command | What it does |
|---|---|---|
| `tempc [LABEL]` | `tempit create [LABEL]` | Create a directory and `cd` into it |
| `tempg [REF]` | `tempit path [REF]` | `cd` into a directory (default: the latest) |
| `templ` | `tempit` or `tempit list` | List directories; `▶` marks the one you are in |
| `temprm REF...` | `tempit remove REF...` | Delete directories |
| `tempsave [REF] [DEST]` | `tempit save [REF] [DEST]` | Move a directory somewhere permanent (default: the one you are in) |
| `tempclean [-y]` | `tempit clean [-y]` | Delete all directories, after showing them and asking |

A `REF` is an id (`3`), a label (`bugfix`), or `.` for the directory you are in (from anywhere
inside it). The commands print paths, so they also compose in scripts:
`cp report.txt "$(tempit path)"`.

When something goes wrong, tempit says what to do about it:

```console
$ tempc api
error: label 'api' is already used by 1-api
  tip: pick another label, or go there with `tempg api`
$ tempg aip
error: no directory matches label 'aip'
  tip: did you mean 'api'?
```

## How it works

- **One private folder.** Directories live in `$TMPDIR/tempit-<uid>` (usually
  `/tmp/tempit-1000`), created with `0700` permissions. tempit refuses to use it if another
  user owns it. Like everything in `/tmp`, it is typically cleared at reboot: save what you
  want to keep.
- **The folder is the database.** A directory is tracked because it is there and named
  `<id>` or `<id>-<label>`. There is no index file that can go stale.
- **Stable ids.** Ids never shift: removing `2` leaves `3` as `3`. The next id is the highest
  one plus one, so numbers stay small and restart at `1` once everything is cleaned.
- **Labels** are unique, so a label always designates one directory. They may contain
  letters, digits, `-`, `_` and `.` (no spaces; `tempc "my project"` suggests `my-project`),
  and cannot be only digits, which are reserved for ids.
- **Saving** moves the directory to `DEST`, like `mv`: into `DEST` if it is an existing
  directory, otherwise to that path. Without `DEST`, it goes to `~/tempit/<label>` (or
  `~/tempit/tempit-<id>` when unlabelled). It never overwrites anything and works across
  filesystems, keeping permissions, timestamps and symlinks.

| Environment variable | Default | Purpose |
|---|---|---|
| `TEMPIT_ROOT` | `$TMPDIR/tempit-<uid>` | Where directories are created |
| `TEMPIT_SAVE_DIR` | `~/tempit` | Where `tempit save` moves directories |

## Upgrading from 1.x

Version 2 is a rewrite in Rust, published on crates.io as `tempit`. It replaces the
`tempit-manager` Python package, so run `pip uninstall tempit-manager` after installing it.

- The shell functions keep their names, and `tempsave` is new. `clean-all` became `clean` and
  now asks for confirmation (`--yes` skips it).
- Directories created by 1.x (`/tmp/<prefix>_<random>`, tracked in `/tmp/tempit_dirs.json`) are
  not picked up. They disappear at the next reboot, or you can delete them by hand.

## Development

```bash
cargo test                                   # unit, CLI and shell integration tests
cargo clippy --all-targets -- -D warnings
cargo fmt
```

The zsh tests run only when `zsh` is installed (CI installs it).

To release, bump `version` in `Cargo.toml`, then push a matching `vX.Y.Z` tag. The release
workflow builds the binaries, creates the GitHub release and publishes to crates.io through
[trusted publishing](https://crates.io/docs/trusted-publishing). Trusted publishing needs the
crate to exist, so the very first version must be published by hand with `cargo publish`.
Then add this repository and the `release.yml` workflow (environment `crates-io`) as a trusted
publisher in the crate's settings on crates.io.

## License

[MIT](LICENSE)
