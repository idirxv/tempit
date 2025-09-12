# Tempit - Temporary Directory Manager

Tempit is a command-line utility and shell helper that lets you create, track, and jump to temporary directories without losing them.
## Features

- Create temporary directories with optional prefixes.
- List tracked directories with size, creation time, age, and file counts.
- Jump to a directory by its number.
- Remove individual directories or clean them all.
- Works via shell integration.

## Installation

### Using pip

```bash
pip install tempit-manager
```

### Shell integration

Add the following line to your shell startup file (`~/.bashrc` or `~/.zshrc`):

```bash
# Bash
eval "$(tempit --init bash)"

# Zsh
eval "$(tempit --init zsh)"
```

## Usage

### CLI commands

```bash
tempit --create [prefix]
tempit --list
tempit --remove <number>
tempit --clean-all
```

### Aliases (after shell init)

| Alias | Description |
|-------|-------------|
| `tempc [prefix]` | Create a new temporary directory and cd into it |
| `tempg <number>` | Jump to a directory by its number |
| `templ` | List tracked temporary directories |
| `temprm <number>` | Remove a tracked temporary directory by its number |
| `tempclean` | Remove all tracked temporary directories |

## License

[MIT](LICENSE)
