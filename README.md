# tempit

[![Crates.io](https://img.shields.io/crates/v/tempit.svg)](https://crates.io/crates/tempit)
[![GitHub Release](https://img.shields.io/github/v/release/idirxv/tempit)](https://github.com/idirxv/tempit/releases/latest)
[![Rust](https://img.shields.io/badge/rust-2024-orange)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

CLI and shell helper to create, track, and jump into temporary working directories.

## Install

From crates.io:

```bash
cargo install tempit
```

With [cargo-binstall](https://github.com/cargo-bins/cargo-binstall):

```bash
cargo binstall tempit
```

Prebuilt binaries are available on the [releases page](https://github.com/idirxv/tempit/releases/latest), or you can build from source with `cargo install --path .`.

## Shell integration

Enables the short aliases (`tempc`, `tempg`, `templ`, `temprm`, `tempclean`) and tab completion.

Bash (`~/.bashrc`) or Zsh (`~/.zshrc`):

```bash
eval "$(tempit init bash)"  # or: zsh
```

Fish (`~/.config/fish/config.fish`):

```fish
tempit init fish | source
```

## Usage

### Shell aliases

| Command | Action |
|---------|--------|
| `tempc [prefix]` | Create a temp directory (default prefix: `tempit`) and `cd` into it |
| `tempg [n\|prefix]` | Jump to a tracked directory by number, prefix, or the most recent if omitted |
| `templ` | List tracked directories with size, age, and file/dir counts |
| `temprm <n>` | Remove a tracked directory by number |
| `tempclean` | Remove all tracked directories |

### CLI

```bash
tempit create [prefix]
tempit list
tempit remove <n>
tempit clean-all
tempit init <bash|zsh|fish>
tempit --version
```

## Configuration

Tracked directories are stored in `<temp_dir>/tempit_dirs.json` by default (e.g. `/tmp/tempit_dirs.json`). Override the path with `TEMPIT_STORAGE_FILE`.

## License

[MIT](LICENSE)
