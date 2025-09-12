"""Core module for the tempit application."""

import logging
from pathlib import Path

from tempit.render import DirectoryRenderer
from tempit.services import DirectoryService
from tempit.storage import DirectoryStorage


class TempitManager:
    """Main manager class for temporary directory operations."""

    def __init__(self, storage_file: Path = Path("/tmp/tempit_dirs.json")):
        """Initialize the TempitManager with dependency injection."""
        self.logger = logging.getLogger(__name__)

        self.storage = DirectoryStorage(storage_file)
        self.service = DirectoryService(storage_file.parent)
        self.renderer = DirectoryRenderer(self.storage, self.service)

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
        directories = self.storage.get_existing_directories()
        self.renderer.render_directory_list(directories)

    def clean_all_directories(self) -> None:
        """Remove all tracked temporary directories."""
        directories = self.storage.get_existing_directories()

        if not directories:
            self.logger.warning("No temporary directories found.")
            return

        removed_count = 0
        for dir_info in directories:
            try:
                if self.service.remove_directory(dir_info.path):
                    removed_count += 1
            except (IOError, OSError) as e:
                self.logger.error("Error removing directory %s: %s", dir_info.path, e)

        self.storage.clear_all()
        self.logger.info("Removed %s temporary directories.", removed_count)
