# Architecture Clarity — Design Spec

**Date:** 2026-03-23
**Goal:** Fix leaky abstractions, hidden side effects, and tangled responsibilities across the tempit codebase. No new features — clarity only.

---

## 1. Problem Statement

The current architecture has four concrete issues:

1. **`DirectoryRenderer` holds dependencies it doesn't use.** It receives both `DirectoryStorage` and `DirectoryService` in its constructor, calls `service.calculate_directory_stats()` during rendering, and never touches `storage` at all. The renderer does computation, not just display.

2. **`get_existing_directories()` has a hidden write side effect.** The method name implies a read, but it silently prunes stale entries and rewrites storage. Callers have no idea.

3. **`DirectoryService` does two unrelated things.** It handles filesystem operations (create/remove directories) and stats calculation (size, file count, age). These have nothing in common beyond operating on paths.

4. **Dev dependencies are listed as runtime dependencies** in `pyproject.toml` (`pytest`, `flake8`, `ruff`, `mypy`, `pylint`).

Two additional minor issues:
- `storage.remove_directory()` always returns `True`, which is misleading.
- `clean_all_directories()` calls `storage.clear_all()` even when some filesystem removals failed.

---

## 2. Chosen Approach: Clean Layer Separation

Establish a clear, one-directional data flow:

```
storage.get_all() + prune_stale()
  → TempitManager computes stats
  → renderer receives (info, stats) pairs and displays
```

`TempitManager` is the single coordinator. Each other class has exactly one job.

---

## 3. Architecture

### Data Flow (after)

```
cli.py
  └─ TempitManager (coordinator)
       ├─ DirectoryStorage    — read/write JSON, no side effects on reads
       ├─ DirectoryService    — filesystem ops only (create, remove)
       ├─ calculate_stats()   — pure function in stats.py
       └─ DirectoryRenderer   — display only, no deps, receives data
```

### Module Responsibilities

| Module | Responsibility | Dependencies |
|---|---|---|
| `models.py` | `DirectoryInfo`, `DirectoryStats` dataclasses | none |
| `storage.py` | JSON persistence | `models` |
| `services.py` | Filesystem create/remove | `models` |
| `stats.py` | Stats calculation (pure function) | `models`, `humanize` |
| `render.py` | Rich table output | `models`, `rich` |
| `core.py` | Orchestration | all of the above |
| `cli.py` | CLI commands | `core` |

---

## 4. Changes Per Module

### `storage.py`

- **Rename** `get_existing_directories()` → `get_all_directories()` — pure read, returns all stored entries without filtering or writing.
- **Add** `prune_stale() -> None` — explicit method that removes entries whose paths no longer exist on the filesystem and writes back.
- **Change** `remove_directory()` return type from `bool` to `None` — it either succeeds or raises; returning `True` unconditionally was misleading.

### `services.py`

- **Remove** `calculate_directory_stats()`, `_get_directory_size()`, `_count_directory_contents()` — these move to `stats.py`.
- `DirectoryService` retains only `create_temp_directory()` and `remove_directory()`.

### `stats.py` *(new file)*

```python
def calculate_stats(dir_info: DirectoryInfo) -> DirectoryStats | None:
    ...
```

Pure function. No class, no state, no logger dependency. Takes a `DirectoryInfo`, returns a `DirectoryStats` or `None` if the path doesn't exist.

### `render.py`

- **Remove** `storage` and `service` from `__init__` — no constructor dependencies at all.
- **Change** `render_directory_list()` signature:
  ```python
  # Before
  def render_directory_list(self, directories: List[DirectoryInfo]) -> None

  # After
  def render_directory_list(self, entries: list[tuple[DirectoryInfo, DirectoryStats]], title: str = ...) -> None
  ```
- Renderer is now a pure display class: construct, call, done.

### `core.py`

- **`print_directories()`**: calls `storage.prune_stale()`, then `storage.get_all_directories()`, computes stats via `calculate_stats()` for each, passes `(info, stats)` pairs to renderer.
- **`clean_all_directories()`**: only calls `storage.remove_directory(path)` for entries that were successfully removed from the filesystem (fixes the clear-all-even-on-failure bug).
- **Constructor**: `DirectoryService` no longer needs `storage_file.parent` — it uses `/tmp` as default directly, removing the implicit path coupling.

### `pyproject.toml`

Move `pytest`, `flake8`, `ruff`, `mypy`, `pylint` from `[project.dependencies]` to `[tool.poetry.group.dev.dependencies]`.

---

## 5. Error Handling

- `storage.remove_directory()` returns `None` — raises `IOError`/`OSError` on failure, which callers already handle.
- `clean_all_directories()` in `TempitManager` tracks which directories were actually removed from the filesystem before calling `storage.remove_directory()` on each, rather than calling `storage.clear_all()` at the end regardless.
- `calculate_stats()` returns `None` if the path doesn't exist — same contract as before.

---

## 6. Testing

- **`DirectoryRenderer`**: no constructor deps — instantiate directly, call `render_directory_list(pairs)`, assert output via `capsys`. No mocking needed.
- **`calculate_stats()`**: pure function — test with a real `tmp_path` fixture, assert returned `DirectoryStats` fields.
- **`TempitManager`**: inject mock/real storage + service as today, but with cleaner seams since renderer no longer needs service injected.
- Existing tests in `test_core.py` should pass with minor updates to reflect the new method names (`get_all_directories` vs `get_existing_directories`).

---

## 7. Out of Scope

- No new features.
- No changes to `cli.py` commands or shell integration (`init.sh`).
- No changes to `models.py`.
- No changes to the JSON storage format or file location.
