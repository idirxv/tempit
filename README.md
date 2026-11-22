# Tempit — Temporary Directory Manager

[![Crates.io](https://img.shields.io/crates/v/tempit.svg)](https://crates.io/crates/tempit)
[![GitHub Release](https://img.shields.io/github/v/release/idirxv/tempit)](https://github.com/idirxv/tempit/releases/latest)
[![Rust](https://img.shields.io/badge/rust-2024-orange)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Fast CLI + shell helper written in Rust to create, track, and jump into temporary directories without losing them.

## Install

```bash
cargo install --path .
```

Or build the release binary directly:

```bash
cargo build --release
cp target/release/tempit ~/.local/bin/tempit
```

Add shell integration to `~/.bashrc` or `~/.zshrc`:

```bash
eval "$(tempit init bash)"   # or: zsh
```

Or for Fish (`~/.config/fish/config.fish`):

```fish
tempit init fish | source
```

## Usage

After shell init, use the aliases:

| Alias | Action |
|-------|--------|
| `tempc [prefix]` | Create temp dir and `cd` into it |
| `tempg <n>` | Jump to tracked dir by number |
| `templ` | List tracked dirs (size, age, file count) |
| `temprm <n>` | Remove tracked dir by number |
| `tempclean` | Remove all tracked dirs |

Raw CLI:

```bash
tempit create [prefix]
tempit list
tempit remove <n>
tempit clean-all
tempit init <shell>
tempit --version
```

Tracked metadata lives at `/tmp/tempit_dirs.json` by default (configurable via `TEMPIT_STORAGE_FILE`).

## License

[MIT](LICENSE)
