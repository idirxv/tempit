"""Storage layer for persisting directory information in JSON format."""

import json
import logging
from pathlib import Path
from typing import List

from tempit.models import DirectoryInfo


class DirectoryStorage:
    """Handles JSON-based persistence of directory information."""

    def __init__(self, storage_file: Path = Path("/tmp/tempit_dirs.json")):
        """Initialize the storage with a JSON file path."""
        self.storage_file = storage_file
        self.logger = logging.getLogger(__name__)
        self._ensure_storage_file()

    def _ensure_storage_file(self) -> None:
        """Ensure the storage file and its parent directory exist."""
        self.storage_file.parent.mkdir(parents=True, exist_ok=True)
        if not self.storage_file.exists():
            self._write_directories([])

    def _read_directories(self) -> List[DirectoryInfo]:
        """Read all directories from the JSON storage file."""
        try:
            with open(self.storage_file, "r", encoding="utf-8") as f:
                data = json.load(f)
                return [DirectoryInfo.from_dict(item) for item in data]
        except (FileNotFoundError, json.JSONDecodeError) as e:
            self.logger.warning("Error reading storage file: %s", e)
            return []

    def _write_directories(self, directories: List[DirectoryInfo]) -> None:
        """Write all directories to the JSON storage file."""
        try:
            data = [dir_info.to_dict() for dir_info in directories]
            with open(self.storage_file, "w", encoding="utf-8") as f:
                json.dump(data, f, indent=2, default=str)
        except (IOError, TypeError) as e:
            self.logger.error("Error writing to storage file: %s", e)
            raise

    def add_directory(self, directory_info: DirectoryInfo) -> None:
        """Add a new directory to storage."""
        directories = self._read_directories()
        directories.append(directory_info)
        self._write_directories(directories)

    def get_existing_directories(self) -> List[DirectoryInfo]:
        """Get only directories that still exist on the filesystem."""
        all_dirs = self._read_directories()
        existing_dirs = [d for d in all_dirs if d.path.exists()]

        # Update storage if some directories no longer exist
        if len(existing_dirs) != len(all_dirs):
            self._write_directories(existing_dirs)
            self.logger.info("Refreshed directories list.")

        return existing_dirs

    def get_path_by_number(self, number: int) -> Path | None:
        """Get the path of a tracked temporary directory by its number."""
        directories = self.get_existing_directories()
        if not directories or not 1 <= number <= len(directories):
            self.logger.error("Invalid directory number: %s", number)
            return None
        return directories[number - 1].path

    def remove_directory(self, path: Path) -> bool:
        """Remove a directory from storage by path."""
        directories = self._read_directories()
        directories = [d for d in directories if d.path != path]
        self._write_directories(directories)
        return True

    def find_directory_by_path(self, path: Path) -> DirectoryInfo | None:
        """Find a directory by its path."""
        directories = self._read_directories()
        for directory in directories:
            if directory.path == path:
                return directory
        return None

    def clear_all(self) -> None:
        """Remove all directories from storage."""
        self._write_directories([])
