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
