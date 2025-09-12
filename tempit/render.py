"""Render directory information from JSON storage."""

# mypy: disable-error-code="arg-type"
import datetime
from typing import List, Optional

import humanize
from tabulate import tabulate  # type: ignore[import-untyped]
from termcolor import colored

from tempit.models import DirectoryInfo, DirectoryStats
from tempit.services import DirectoryService
from tempit.storage import DirectoryStorage


class DirectoryRenderer:
    """Handles rendering of directory information from JSON storage."""

    def __init__(self, storage: DirectoryStorage, service: DirectoryService):
        """Initialize the renderer with storage and service dependencies."""
        self.storage = storage
        self.service = service

    def get_headers(self) -> List[str]:
        """Return formatted table headers for directory listings."""
        return [
            colored("#", "white", attrs=["bold"]),
            colored("Name", "white", attrs=["bold"]),
            colored("Path", "white", attrs=["bold"]),
            colored("Size", "white", attrs=["bold"]),
            colored("Created", "white", attrs=["bold"]),
            colored("Age", "white", attrs=["bold"]),
            colored("Contents", "white", attrs=["bold"]),
        ]

    def render_directory_list(
        self, directories: List[DirectoryInfo], title: str = "Temporary Directories"
    ) -> None:
        """Render all directories from JSON storage."""

        if not directories:
            print(colored("No temporary directories found.", "yellow"))
            return

        table_data = []
        for i, dir_info in enumerate(directories):
            stats = self.service.calculate_directory_stats(dir_info)
            if not stats:
                continue
            friendly_name = dir_info.prefix
            row = self._create_table_row(dir_info, stats, i, friendly_name)
            table_data.append(row)

        self._print_table(table_data, self.get_headers(), title)

    def _create_table_row(
        self,
        dir_info: DirectoryInfo,
        stats: DirectoryStats,
        index: int,
        friendly_name: str,
    ) -> List[str]:
        """Create a formatted table row from directory info and stats."""
        # Format creation date
        created_str = dir_info.created.strftime("%Y-%m-%d %H:%M")

        # Name column with color
        name = colored(friendly_name, "cyan", attrs=["bold"])

        # Size with color thresholds
        size_color = "green"
        if stats.size_bytes > 10 * 1024 * 1024:  # > 10MB
            size_color = "yellow"
        if stats.size_bytes > 100 * 1024 * 1024:  # > 100MB
            size_color = "red"
        human_size = colored(stats.human_size, size_color)

        # Age with color
        age_color = "green"
        if "day" in stats.age or "month" in stats.age or "year" in stats.age:
            age_color = "yellow"
        colored_age = colored(stats.age, age_color)

        # Contents info
        contents_info = (
            f"{colored(stats.file_count, 'blue')} files, "
            f"{colored(stats.dir_count, 'blue')} dirs"
        )

        return [
            colored(str(index + 1), "white", attrs=["bold"]),
            name,
            str(dir_info.path),
            human_size,
            created_str,
            colored_age,
            contents_info,
        ]

    def _print_table(
        self, rows: List[List[str]], hdrs: List[str], title: Optional[str] = None
    ) -> None:
        """Print a nicely formatted table."""
        if title:
            print()
            print(colored(title, "white", attrs=["bold"]))
        print(
            tabulate(
                rows,
                headers=hdrs,
                tablefmt="rounded_grid",
                numalign="center",
            )
        )
        print()


# Legacy functions for backward compatibility
def headers() -> List[str]:
    """Return formatted table headers for directory listings."""
    return [
        colored("#", "white", attrs=["bold"]),
        colored("Name", "white", attrs=["bold"]),
        colored("Path", "white", attrs=["bold"]),
        colored("Size", "white", attrs=["bold"]),
        colored("Created", "white", attrs=["bold"]),
        colored("Age", "white", attrs=["bold"]),
        colored("Contents", "white", attrs=["bold"]),
    ]


def row_from_info(info: dict, index: int, friendly_name: str) -> List[str]:
    """Create a formatted row from directory info (legacy function).

    Expected keys in info: path, human_size, created (datetime), file_count, dir_count, size
    """
    created: datetime.datetime = info["created"]
    created_str = created.strftime("%Y-%m-%d %H:%M")

    # Name column
    name = colored(friendly_name, "cyan", attrs=["bold"])

    # Size with color thresholds
    size_color = "green"
    if info["size"] > 10 * 1024 * 1024:  # > 10MB
        size_color = "yellow"
    if info["size"] > 100 * 1024 * 1024:  # > 100MB
        size_color = "red"
    human_size = colored(info["human_size"], size_color)  # type: ignore[arg-type]

    # Age
    age = humanize.naturaltime(datetime.datetime.now() - created)
    age_color = "green"
    if "day" in age or "month" in age or "year" in age:
        age_color = "yellow"
    colored_age = colored(age, age_color)  # type: ignore[arg-type]

    # Contents
    contents_info = (
        f"{colored(info['file_count'], 'blue')} files, "
        f"{colored(info['dir_count'], 'blue')} dirs"
    )

    return [
        colored(str(index + 1), "white", attrs=["bold"]),
        name,
        info["path"],
        human_size,
        created_str,
        colored_age,
        contents_info,
    ]


def print_table(rows: List[List[str]], hdrs: List[str], title: Optional[str] = None) -> None:
    """Print a nicely formatted table of directories."""
    if title:
        print()
        print(colored(title, "white", attrs=["bold"]))
    print(
        tabulate(
            rows,
            headers=hdrs,
            tablefmt="rounded_grid",
            numalign="center",
        )
    )
    print()
