# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

tempit-manager is a CLI utility and shell helper for creating, tracking, and jumping to temporary directories, built with Poetry.

## Commands

```bash
# Install dependencies
poetry install

# Run all tests
poetry run pytest .

# Run the linters
poetry run flake8 . --max-line-length=127 --max-complexity=10 --statistics
poetry run ruff check .
poetry run mypy --python-version 3.12 .
poetry run pylint --py-version 3.12 tempit tests
```

## Architecture

```
CLI (cli.py) → TempitManager (core.py)
                  ├── DirectoryStorage (storage.py)  — JSON persistence at /tmp/tempit_dirs.json
                  ├── DirectoryService (services.py)  — temp dir creation/removal only
                  ├── calculate_stats() (stats.py)    — pure function: DirectoryInfo → DirectoryStats | None
                  └── DirectoryRenderer (render.py)   — rich table output, accepts (DirectoryInfo, DirectoryStats) pairs
```

- **models.py**: `DirectoryInfo` (path, creation time, prefix; JSON-serializable) and `DirectoryStats` (size, counts, age).
- **stats.py**: Pure `calculate_stats(dir_info)` function — no side effects, returns `None` for missing paths.
- **storage.py**: `get_all_directories()` is a pure read; `prune_stale()` is the sole method that removes stale entries.
- **shell/init.sh**: Bash/Zsh integration providing `tempc`, `tempg`, `templ`, `temprm`, `tempclean` aliases that wrap the CLI.
- **Entry point**: `tempit.cli:main`, registered as console script `tempit`.

## Key Details

- Tool is used via CLI and shell aliases.


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