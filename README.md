# Tempit - Temporary Directory Manager

Tempit is a command-line utility and shell helper that lets you create, track, and jump to temporary directories without losing them.
## Features

- Create temporary directories with optional prefixes.
- List tracked directories with size, creation time, age, and file counts.
- Jump to a directory by its number.
- Remove individual directories or clean them all.
- Search directories by text.
- Works via shell integration.

## Installation

### Using pip

```bash
pip install tempit
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
tempit --get <number>
tempit --remove <number>
tempit --search <term>
tempit --clean-all
```

### Aliases (after shell init)

| Alias | Description |
|-------|-------------|
| `tempc [prefix]` | Create a new temporary directory and cd into it |
| `tempg <number>` | Change into the directory by its number |
| `templ` | List tracked temporary directories |

## License

[MIT](LICENSE)
