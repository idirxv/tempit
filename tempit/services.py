"""Service layer for directory operations and statistics."""

import logging
import shutil
import tempfile
from datetime import datetime
from pathlib import Path
from typing import Tuple

import humanize

from tempit.models import DirectoryInfo, DirectoryStats


class DirectoryService:
    """Service for directory operations and statistics calculation."""

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

    def calculate_directory_stats(self, directory_info: DirectoryInfo) -> DirectoryStats | None:
        """Calculate runtime statistics for a directory."""
        dir_path: Path = directory_info.path
        if not dir_path.exists():
            self.logger.warning("Directory does not exist: %s", dir_path)
            return None

        # Calculate size
        total_size = self._get_directory_size(dir_path)
        human_size = humanize.naturalsize(total_size, binary=True)

        # Count files and directories
        file_count, dir_count = self._count_directory_contents(dir_path)

        # Calculate age
        creation_time = directory_info.created
        age = humanize.naturaltime(datetime.now() - creation_time)

        return DirectoryStats(
            size_bytes=total_size,
            human_size=human_size,
            file_count=file_count,
            dir_count=dir_count,
            age=age,
        )

    @staticmethod
    def _get_directory_size(directory: Path) -> int:
        """Get the total size of a directory in bytes."""
        return sum(f.stat().st_size for f in directory.rglob("*") if f.is_file())

    @staticmethod
    def _count_directory_contents(directory: Path) -> Tuple[int, int]:
        """Count files and subdirectories in a directory."""
        files = sum(1 for f in directory.rglob("*") if f.is_file())
        dirs = sum(1 for d in directory.rglob("*") if d.is_dir())
        return files, dirs
