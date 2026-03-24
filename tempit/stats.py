"""Pure stats calculation for directory information."""

from datetime import datetime

import humanize

from tempit.models import DirectoryInfo, DirectoryStats


def calculate_stats(dir_info: DirectoryInfo) -> DirectoryStats | None:
    """Calculate stats for a directory. Returns None if the path doesn't exist."""
    dir_path = dir_info.path
    if not dir_path.exists():
        return None

    all_entries = list(dir_path.rglob("*"))
    files = [f for f in all_entries if f.is_file()]
    dirs = [d for d in all_entries if d.is_dir()]
    total_size = sum(f.stat().st_size for f in files)
    file_count = len(files)
    dir_count = len(dirs)

    return DirectoryStats(
        size_bytes=total_size,
        human_size=humanize.naturalsize(total_size, binary=True),
        file_count=file_count,
        dir_count=dir_count,
        age=humanize.naturaltime(datetime.now() - dir_info.created),
    )
