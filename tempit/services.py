"""Service layer for directory operations."""

import logging
import secrets
import shutil
from datetime import datetime
from pathlib import Path

from tempit.models import DirectoryInfo


class DirectoryService:
    """Service for directory operations."""

    def __init__(self, temp_base_dir: Path = Path("/tmp")):
        """Initialize the directory service."""
        self.temp_base_dir = temp_base_dir
        self.logger = logging.getLogger(__name__)

    def create_temp_directory(self, prefix: str) -> DirectoryInfo:
        """Create a new temporary directory and return its info."""
        try:
            unique_name = f"{prefix}_{secrets.token_hex(4)}"
            temp_dir = self.temp_base_dir / unique_name
            temp_dir.mkdir(parents=True, exist_ok=False)
            return DirectoryInfo(
                path=temp_dir, created=datetime.now(), prefix=prefix
            )
        except (IOError, OSError) as e:
            self.logger.error("Error creating temporary directory: %s", e)
            raise

    def remove_directory(self, path: Path) -> bool:
        """Remove a directory from the filesystem."""
        try:
            if path.exists():
                shutil.rmtree(path)
                self.logger.info("Removed directory: %s", path)
                return True
            self.logger.warning("Directory does not exist: %s", path)
            return False
        except (IOError, OSError) as e:
            self.logger.error("Error removing directory %s: %s", path, e)
            return False
