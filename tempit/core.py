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

    def create(self, prefix: str) -> Path:
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
        all_entries = [(d, calculate_stats(d)) for d in directories]
        entries = [(d, s) for d, s in all_entries if s is not None]
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
