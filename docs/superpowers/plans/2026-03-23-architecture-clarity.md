# Architecture Clarity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix leaky abstractions, hidden side effects, and tangled responsibilities in tempit-manager — no new features, clarity only.

**Architecture:** `TempitManager` becomes the single coordinator: it fetches directories, computes stats, and passes `(DirectoryInfo, DirectoryStats)` pairs to a dependency-free renderer. Stats calculation moves to a pure function in `stats.py`. Storage read methods lose their hidden write side effects.

**Tech Stack:** Python 3.10+, Poetry, pytest, rich, humanize, typer

---

## File Map

| Action | File | Change |
|--------|------|--------|
| Create | `tempit/stats.py` | Pure `calculate_stats()` function (moved from services.py) |
| Create | `tests/test_stats.py` | Tests for `calculate_stats()` |
| Create | `tests/test_render.py` | Tests for `DirectoryRenderer` |
| Modify | `tempit/storage.py` | Rename method, add `prune_stale()`, update internals, fix return type |
| Modify | `tempit/services.py` | Remove stats methods |
| Modify | `tempit/render.py` | Remove constructor deps, change `render_directory_list()` signature |
| Modify | `tempit/core.py` | Orchestration changes throughout |
| Modify | `tests/test_core.py` | Update 3 storage call sites |
| Modify | `pyproject.toml` | Move dev deps to group |

---

## Task 1: Create `stats.py` — pure stats function

**Files:**
- Create: `tempit/stats.py`
- Create: `tests/test_stats.py`

- [ ] **Step 1: Write the failing test**

Create `tests/test_stats.py`:

```python
from pathlib import Path
import pytest
from tempit.models import DirectoryInfo, DirectoryStats
from tempit.stats import calculate_stats
from datetime import datetime


@pytest.fixture
def dir_info(tmp_path):
    return DirectoryInfo(path=tmp_path, created=datetime.now(), prefix="test")


def test_calculate_stats_returns_stats_for_existing_dir(dir_info):
    stats = calculate_stats(dir_info)
    assert stats is not None
    assert isinstance(stats, DirectoryStats)
    assert stats.size_bytes >= 0
    assert stats.file_count >= 0
    assert stats.dir_count >= 0
    assert stats.human_size != ""
    assert stats.age != ""


def test_calculate_stats_returns_none_for_missing_dir(tmp_path):
    missing = DirectoryInfo(path=tmp_path / "nonexistent", created=datetime.now(), prefix="test")
    assert calculate_stats(missing) is None


def test_calculate_stats_counts_files(dir_info):
    (dir_info.path / "file.txt").write_text("hello")
    stats = calculate_stats(dir_info)
    assert stats.file_count == 1
    assert stats.size_bytes == 5
```

- [ ] **Step 2: Run test to confirm it fails**

```bash
poetry run pytest tests/test_stats.py -v
```

Expected: `ImportError` or `ModuleNotFoundError` — `tempit.stats` doesn't exist yet.

- [ ] **Step 3: Create `tempit/stats.py`**

```python
"""Pure stats calculation for directory information."""

from datetime import datetime
from pathlib import Path

import humanize

from tempit.models import DirectoryInfo, DirectoryStats


def calculate_stats(dir_info: DirectoryInfo) -> DirectoryStats | None:
    """Calculate stats for a directory. Returns None if the path doesn't exist."""
    dir_path = dir_info.path
    if not dir_path.exists():
        return None

    total_size = sum(f.stat().st_size for f in dir_path.rglob("*") if f.is_file())
    file_count = sum(1 for f in dir_path.rglob("*") if f.is_file())
    dir_count = sum(1 for d in dir_path.rglob("*") if d.is_dir())

    return DirectoryStats(
        size_bytes=total_size,
        human_size=humanize.naturalsize(total_size, binary=True),
        file_count=file_count,
        dir_count=dir_count,
        age=humanize.naturaltime(datetime.now() - dir_info.created),
    )
```

- [ ] **Step 4: Run tests to confirm they pass**

```bash
poetry run pytest tests/test_stats.py -v
```

Expected: 3 tests pass.

- [ ] **Step 5: Commit**

```bash
git add tempit/stats.py tests/test_stats.py
git commit -m "feat: add calculate_stats pure function in stats.py"
```

---

## Task 2: Update `storage.py` — split read/write concerns

**Files:**
- Modify: `tempit/storage.py`
- Modify: `tests/test_core.py` (update 3 call sites at lines 26, 43, 53)

- [ ] **Step 1: Write failing tests for `prune_stale()` and `get_all_directories()`**

Add to the bottom of `tests/test_core.py`:

```python
def test_get_all_directories_is_pure_read(tempit_manager, tmp_path):
    """get_all_directories() should not remove stale entries from storage."""
    # Create a dir, record it, then delete it from filesystem
    path = tempit_manager.create(prefix="stale")
    path.rmdir()
    # Raw storage should still have the entry
    all_dirs = tempit_manager.storage.get_all_directories()
    assert any(d.path == path for d in all_dirs)


def test_prune_stale_removes_missing_entries(tempit_manager):
    """prune_stale() should remove entries whose paths no longer exist."""
    path = tempit_manager.create(prefix="stale")
    path.rmdir()
    tempit_manager.storage.prune_stale()
    all_dirs = tempit_manager.storage.get_all_directories()
    assert not any(d.path == path for d in all_dirs)
```

- [ ] **Step 2: Run to confirm they fail**

```bash
poetry run pytest tests/test_core.py::test_get_all_directories_is_pure_read tests/test_core.py::test_prune_stale_removes_missing_entries -v
```

Expected: `AttributeError` — `get_all_directories` and `prune_stale` don't exist yet.

- [ ] **Step 3: Update `tempit/storage.py`**

Replace `get_existing_directories` with `get_all_directories`, add `prune_stale`, update `get_path_by_number`, fix `remove_directory` return type:

```python
def get_all_directories(self) -> List[DirectoryInfo]:
    """Read all stored directory entries. Pure read — no side effects."""
    return self._read_directories()

def prune_stale(self) -> None:
    """Remove entries for directories that no longer exist on the filesystem."""
    all_dirs = self._read_directories()
    existing = [d for d in all_dirs if d.path.exists()]
    if len(existing) != len(all_dirs):
        self._write_directories(existing)
        self.logger.info("Pruned %d stale entries.", len(all_dirs) - len(existing))

def get_path_by_number(self, number: int) -> Path | None:
    """Get the path of a tracked directory by its 1-based index."""
    directories = self.get_all_directories()
    if not directories or not 1 <= number <= len(directories):
        self.logger.error("Invalid directory number: %s", number)
        return None
    return directories[number - 1].path

def remove_directory(self, path: Path) -> None:
    """Remove a directory entry from storage by path. Raises on write failure."""
    directories = self._read_directories()
    self._write_directories([d for d in directories if d.path != path])
```

Also delete the old `get_existing_directories` method and `find_directory_by_path` if unused (check: `find_directory_by_path` is not called anywhere in the codebase — delete it).

- [ ] **Step 4: Update the 3 call sites in `tests/test_core.py`**

Change every `storage.get_existing_directories()` to `storage.get_all_directories()`:

- Line 26: `tempit_manager.storage.get_existing_directories()[0].path` → `tempit_manager.storage.get_all_directories()[0].path`
- Line 43: `tempit_manager.storage.get_existing_directories()[0].path` → `tempit_manager.storage.get_all_directories()[0].path`
- Line 53: `tempit_manager.storage.get_existing_directories()` → `tempit_manager.storage.get_all_directories()`

- [ ] **Step 5: Run all storage-related tests**

```bash
poetry run pytest tests/test_core.py -v
```

Expected: all existing tests + 2 new tests pass.

- [ ] **Step 6: Commit**

```bash
git add tempit/storage.py tests/test_core.py
git commit -m "refactor: split storage read/write — add prune_stale, rename get_all_directories"
```

---

## Task 3: Strip stats from `services.py`

**Files:**
- Modify: `tempit/services.py`

The stats code is now in `stats.py`. Remove it from `services.py`.

- [ ] **Step 1: Delete the three stats methods from `tempit/services.py`**

Remove entirely from `services.py`:
- `calculate_directory_stats()` (lines 48–72)
- `_get_directory_size()` (lines 74–77)
- `_count_directory_contents()` (lines 79–84)

Also remove the now-unused imports at the top of `services.py`:
- Remove `from typing import Tuple`
- Keep `from datetime import datetime` only if still used (it's not after removing stats) — remove it too
- Keep `humanize` import only if still used — it's not, remove it

Final `services.py` imports section:
```python
import logging
import shutil
import tempfile
from pathlib import Path

from tempit.models import DirectoryInfo
```

- [ ] **Step 2: Run all tests**

```bash
poetry run pytest . -v
```

Expected: all tests still pass (nothing calls service stats methods directly in tests).

- [ ] **Step 3: Commit**

```bash
git add tempit/services.py
git commit -m "refactor: remove stats calculation from DirectoryService"
```

---

## Task 4: Update `render.py` — pure display class

**Files:**
- Modify: `tempit/render.py`
- Create: `tests/test_render.py`

- [ ] **Step 1: Write failing tests for the new renderer interface**

Create `tests/test_render.py`:

```python
import sys
from datetime import datetime
from pathlib import Path

import pytest
from tempit.models import DirectoryInfo, DirectoryStats
from tempit.render import DirectoryRenderer


@pytest.fixture
def renderer():
    return DirectoryRenderer()


@pytest.fixture
def sample_entries(tmp_path):
    info = DirectoryInfo(path=tmp_path, created=datetime.now(), prefix="test")
    stats = DirectoryStats(
        size_bytes=1024,
        human_size="1.0 KiB",
        file_count=2,
        dir_count=1,
        age="just now",
    )
    return [(info, stats)]


def test_renderer_requires_no_constructor_args():
    """Renderer should be constructable with no arguments."""
    renderer = DirectoryRenderer()
    assert renderer is not None


def test_render_empty_list_prints_message(renderer, capsys):
    renderer.render_directory_list([])
    captured = capsys.readouterr()
    assert "No temporary directories found" in captured.out


def test_render_entries_shows_path(renderer, sample_entries, capsys):
    renderer.render_directory_list(sample_entries)
    captured = capsys.readouterr()
    path_str = str(sample_entries[0][0].path)
    assert path_str in captured.out
```

- [ ] **Step 2: Run to confirm they fail**

```bash
poetry run pytest tests/test_render.py -v
```

Expected: `TypeError` — `DirectoryRenderer()` currently requires `storage` and `service`.

- [ ] **Step 3: Rewrite `tempit/render.py`**

```python
"""Render directory information as a rich table."""

import sys
from typing import List, Tuple

from rich.console import Console
from rich.table import Table

from tempit.models import DirectoryInfo, DirectoryStats


class DirectoryRenderer:
    """Renders (DirectoryInfo, DirectoryStats) pairs as a rich table. No external dependencies."""

    def render_directory_list(
        self,
        entries: List[Tuple[DirectoryInfo, DirectoryStats]],
        title: str = "Temporary Directories",
    ) -> None:
        """Render a list of (info, stats) pairs as a rich table."""
        console = Console(file=sys.stdout)

        if not entries:
            console.print("[yellow]No temporary directories found.[/yellow]")
            return

        table = Table(title=title, show_header=True, header_style="bold white")
        table.add_column("#", justify="center", style="bold white")
        table.add_column("Name", style="bold cyan", no_wrap=True)
        table.add_column("Path")
        table.add_column("Size")
        table.add_column("Created")
        table.add_column("Age")
        table.add_column("Contents")

        for i, (dir_info, stats) in enumerate(entries):
            table.add_row(*self._create_table_row(dir_info, stats, i))

        console.print()
        console.print(table)
        console.print()

    def _create_table_row(
        self,
        dir_info: DirectoryInfo,
        stats: DirectoryStats,
        index: int,
    ) -> List[str]:
        created_str = dir_info.created.strftime("%Y-%m-%d %H:%M")

        if stats.size_bytes > 100 * 1024 * 1024:
            size_markup = f"[red]{stats.human_size}[/red]"
        elif stats.size_bytes > 10 * 1024 * 1024:
            size_markup = f"[yellow]{stats.human_size}[/yellow]"
        else:
            size_markup = f"[green]{stats.human_size}[/green]"

        if "day" in stats.age or "month" in stats.age or "year" in stats.age:
            age_markup = f"[yellow]{stats.age}[/yellow]"
        else:
            age_markup = f"[green]{stats.age}[/green]"

        contents = f"[blue]{stats.file_count}[/blue] files, [blue]{stats.dir_count}[/blue] dirs"

        return [
            str(index + 1),
            dir_info.prefix,
            str(dir_info.path),
            size_markup,
            created_str,
            age_markup,
            contents,
        ]
```

- [ ] **Step 4: Run renderer tests**

```bash
poetry run pytest tests/test_render.py -v
```

Expected: 3 tests pass.

- [ ] **Step 5: Run full test suite (some core tests will fail — expected)**

```bash
poetry run pytest . -v
```

Expected: `test_render.py` and `test_stats.py` pass. `test_core.py` will have failures because `core.py` still calls `DirectoryRenderer(self.storage, self.service)` — that gets fixed in Task 5.

- [ ] **Step 6: Commit**

```bash
git add tempit/render.py tests/test_render.py
git commit -m "refactor: make DirectoryRenderer a pure display class with no constructor deps"
```

---

## Task 5: Update `core.py` — orchestration

**Files:**
- Modify: `tempit/core.py`

- [ ] **Step 1: Rewrite `tempit/core.py`**

```python
"""Core module for the tempit application."""

import logging
from pathlib import Path

from tempit.render import DirectoryRenderer
from tempit.services import DirectoryService
from tempit.stats import calculate_stats
from tempit.storage import DirectoryStorage


class TempitManager:
    """Main manager class for temporary directory operations."""

    def __init__(self, storage_file: Path = Path("/tmp/tempit_dirs.json")):
        """Initialize the TempitManager with dependency injection."""
        self.logger = logging.getLogger(__name__)
        self.storage = DirectoryStorage(storage_file)
        self.service = DirectoryService()
        self.renderer = DirectoryRenderer()

    def init_shell(self, shell: str) -> None:
        """Initialize Tempit in the current shell."""
        if shell in ["bash", "zsh"]:
            init_script_path = Path(__file__).parent / "shell" / "init.sh"
            try:
                with init_script_path.open("r", encoding="utf-8") as f:
                    print(f.read())
            except FileNotFoundError:
                self.logger.error("Error reading initialization script: %s", init_script_path)
        else:
            self.logger.error("Unsupported shell: %s", shell)

    def create(self, prefix: str = "tempit") -> Path:
        """Create a new temporary directory and track it."""
        try:
            dir_info = self.service.create_temp_directory(prefix)
            self.storage.add_directory(dir_info)
            self.logger.info("Created temporary directory: %s", dir_info.path)
            return dir_info.path
        except (IOError, OSError) as e:
            self.logger.error("Error creating temporary directory: %s", e)
            raise

    def remove(self, number: int) -> bool:
        """Remove a tracked temporary directory by its number."""
        try:
            self.storage.prune_stale()
            dir_path = self.storage.get_path_by_number(number)
            if dir_path is None:
                return False
            success = self.service.remove_directory(dir_path)
            if success:
                self.storage.remove_directory(dir_path)
                self.logger.info("Removed temporary directory: %s", dir_path)
                return True
            return False
        except (IOError, OSError) as e:
            self.logger.error("Error removing temporary directory: %s", e)
            return False

    def print_directories(self) -> None:
        """Print a formatted table of tracked temporary directories."""
        self.storage.prune_stale()
        directories = self.storage.get_all_directories()
        entries = [(d, calculate_stats(d)) for d in directories]
        entries = [(d, s) for d, s in entries if s is not None]
        self.renderer.render_directory_list(entries)

    def get_path_by_number(self, number: int) -> Path | None:
        """Return the path for a tracked directory by its number."""
        self.storage.prune_stale()
        return self.storage.get_path_by_number(number)

    def clean_all_directories(self) -> None:
        """Remove all tracked temporary directories."""
        self.storage.prune_stale()
        directories = self.storage.get_all_directories()

        if not directories:
            self.logger.warning("No temporary directories found.")
            return

        removed_count = 0
        for dir_info in directories:
            try:
                if self.service.remove_directory(dir_info.path):
                    self.storage.remove_directory(dir_info.path)
                    removed_count += 1
            except (IOError, OSError) as e:
                self.logger.error("Error removing directory %s: %s", dir_info.path, e)

        self.logger.info("Removed %s temporary directories.", removed_count)
```

- [ ] **Step 2: Run the full test suite**

```bash
poetry run pytest . -v
```

Expected: all tests pass.

- [ ] **Step 3: Commit**

```bash
git add tempit/core.py
git commit -m "refactor: update TempitManager to coordinate stats and use prune_stale explicitly"
```

---

## Task 6: Fix `pyproject.toml` — separate dev dependencies

**Files:**
- Modify: `pyproject.toml`

- [ ] **Step 1: Move dev tools out of `[project.dependencies]`**

Remove `pytest`, `flake8`, `ruff`, `mypy`, `pylint` from the `[project] dependencies` list, and add a new section at the bottom of `pyproject.toml`:

```toml
[tool.poetry.group.dev.dependencies]
pytest = ">=8.4.2,<9.0.0"
flake8 = ">=7.3.0,<8.0.0"
ruff = ">=0.13.0,<0.14.0"
mypy = ">=1.18.1,<2.0.0"
pylint = ">=3.3.8,<4.0.0"
```

The `[project.dependencies]` block should retain only: `humanize`, `rich`, `typer`.

- [ ] **Step 2: Reinstall dependencies and verify**

```bash
poetry install
poetry run pytest . -v
```

Expected: all tests pass; dev tools still available via `poetry run`.

- [ ] **Step 3: Commit**

```bash
git add pyproject.toml
git commit -m "chore: move dev dependencies to poetry dev group"
```

---

## Final Verification

- [ ] **Run full test suite one last time**

```bash
poetry run pytest . -v
```

Expected: all tests pass, no warnings about missing methods.

- [ ] **Smoke test the CLI**

```bash
poetry run tempit create testdir
poetry run tempit list
poetry run tempit remove 1
```

Expected: creates a temp dir, lists it in a table, removes it cleanly.
