"""Render directory information from JSON storage."""

import sys
from typing import List

from rich.console import Console
from rich.table import Table

from tempit.models import DirectoryInfo, DirectoryStats
from tempit.services import DirectoryService
from tempit.storage import DirectoryStorage


class DirectoryRenderer:
    """Handles rendering of directory information from JSON storage."""

    def __init__(self, storage: DirectoryStorage, service: DirectoryService):
        self.storage = storage
        self.service = service

    def render_directory_list(
        self, directories: List[DirectoryInfo], title: str = "Temporary Directories"
    ) -> None:
        """Render all directories as a rich table."""
        console = Console(file=sys.stdout)

        if not directories:
            console.print("[yellow]No temporary directories found.[/yellow]")
            return

        table = Table(title=title, show_header=True, header_style="bold white")
        table.add_column("#", justify="center", style="bold white")
        table.add_column("Name", style="bold cyan", no_wrap=True)
        table.add_column("Path")
        table.add_column("Size")
        table.add_column("Created")
        table.add_column("Age")
        table.add_column("Contents")

        for i, dir_info in enumerate(directories):
            stats = self.service.calculate_directory_stats(dir_info)
            if not stats:
                continue
            table.add_row(*self._create_table_row(dir_info, stats, i))

        console.print()
        console.print(table)
        console.print()

    def _create_table_row(
        self,
        dir_info: DirectoryInfo,
        stats: DirectoryStats,
        index: int,
    ) -> List[str]:
        created_str = dir_info.created.strftime("%Y-%m-%d %H:%M")

        if stats.size_bytes > 100 * 1024 * 1024:
            size_markup = f"[red]{stats.human_size}[/red]"
        elif stats.size_bytes > 10 * 1024 * 1024:
            size_markup = f"[yellow]{stats.human_size}[/yellow]"
        else:
            size_markup = f"[green]{stats.human_size}[/green]"

        if "day" in stats.age or "month" in stats.age or "year" in stats.age:
            age_markup = f"[yellow]{stats.age}[/yellow]"
        else:
            age_markup = f"[green]{stats.age}[/green]"

        contents = f"[blue]{stats.file_count}[/blue] files, [blue]{stats.dir_count}[/blue] dirs"

        return [
            str(index + 1),
            dir_info.prefix,
            str(dir_info.path),
            size_markup,
            created_str,
            age_markup,
            contents,
        ]
