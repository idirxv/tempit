# Tempit - Temporary Directory Manager for Oh My Zsh

Tempit is a small plugin for Oh My Zsh that helps you create, manage, and navigate temporary directories with ease. It provides a persistent tracking system so your temporary directories won't get lost.

## Features

- Create temporary directories with custom prefixes
- List all tracked temporary directories with sizes
- Navigate to temporary directories
- Remove temporary directories

## Installation

### Manual Installation

1. Clone the repository to your Oh My Zsh custom plugins directory:

```bash
git clone https://github.com/idirxv/tempit.git ${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/tempit
```

2. Add the plugin to your `.zshrc` file:

```bash
plugins=(... tempit)
```

3. Reload your shell:

```bash
source ~/.zshrc
```

## Dependencies

Tempit requires Python 3 and the following Python packages (included in requirements.txt):
- humanize
- tabulate
- termcolor

Install dependencies with:

```bash
cd ${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/tempit
pip install -r requirements.txt
```

## Usage

### Commands

| Command | Description |
|---------|-------------|
| `tempc [prefix]` | Create a new temporary directory (optional custom prefix) and cd into it |
| `templ` | List all tracked temporary directories |
| `tempg <number>` | Go to (cd) the temporary directory by its number |
| `tempr <number>` | Remove a temporary directory by its number |

### Examples

```bash
# Create a temporary directory with default prefix
tempc

# Create a temporary directory with custom prefix
tempc foo

# List all tracked temporary directories
templ

# Navigate to temporary directory #2
tempg 2

# Remove temporary directory #1
tempr 1
```

## License

MIT License