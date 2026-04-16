# Tempit — Temporary Directory Manager

[![Codacy Badge](https://app.codacy.com/project/badge/Grade/355fe09860a44384a5efe8580fbfc20a)](https://app.codacy.com/gh/idirxv/tempit/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade)
[![Python](https://img.shields.io/badge/python-3.10%2B-blue)](https://www.python.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

CLI + shell helper to create, track, and jump into temporary directories without losing them.

## Install

```bash
pip install tempit-manager
```

Add shell integration to `~/.bashrc` or `~/.zshrc`:

```bash
eval "$(tempit init bash)"   # or: zsh
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

Tracked metadata lives at `/tmp/tempit_dirs.json`.

## License

[MIT](LICENSE)
