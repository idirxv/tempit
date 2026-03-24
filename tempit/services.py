"""Service layer for directory operations."""

import logging
import shutil
import tempfile
from datetime import datetime
from pathlib import Path

from tempit.models import DirectoryInfo


class DirectoryService:
    """Service for directory operations."""

    def __init__(self, temp_base_dir: Path = Path("/tmp")):
        """Initialize the directory service."""
        self.temp_base_dir = temp_base_dir
        self.logger = logging.getLogger(__name__)

    def create_temp_directory(self, prefix: str = "tempit") -> DirectoryInfo:
        """Create a new temporary directory and return its info."""
        try:
            temp_dir = tempfile.mkdtemp(prefix=f"{prefix}_", dir=self.temp_base_dir)

            return DirectoryInfo(
                path=Path(temp_dir), created=datetime.now(), prefix=prefix
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

