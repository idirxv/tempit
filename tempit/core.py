# pylint: disable=missing-module-docstring
# pylint: disable=unspecified-encoding
import os
import tempfile
import shutil
import datetime
from typing import List
from termcolor import colored
from tabulate import tabulate
import humanize

class TempitManager:
    """ A class to manage temporary directories with persistent tracking """

    def __init__(self, tempit_file: str = "/tmp/tempit_dirs.txt"):
        """ Initialize the TempitManager """
        self.tempit_file = tempit_file
        self.tempit_dir = os.path.dirname(self.tempit_file)

        # Ensure the file exists
        if not os.path.exists(self.tempit_file):
            with open(self.tempit_file, "w") as _:
                pass

    def create(self, prefix: str = "tempit") -> str:
        """ Create a new temporary directory and track it """
        try:
            temp_dir = tempfile.mkdtemp(prefix=prefix + "_", dir=self.tempit_dir)

            # Append directory path to tracking file
            with open(self.tempit_file, "a") as file:
                file.write(f"{temp_dir}\n")

            print(temp_dir)
            return temp_dir

        except (IOError, OSError) as e:
            print(colored(f"Error creating temporary directory: {e}", "red"))
            raise

    def _get_directory_size(self, directory: str) -> tuple:
        """ Get the size of a directory in bytes. """
        total_size = 0
        for dirpath, _, filenames in os.walk(directory):
            for f in filenames:
                fp = os.path.join(dirpath, f)
                total_size += os.path.getsize(fp)
        return total_size, humanize.naturalsize(total_size, binary=True)

    def _get_directory_info(self, directory: str) -> dict:
        """ Get comprehensive information about a directory """
        # Get the directory size
        total_size, human_size = self._get_directory_size(directory)

        # Get creation time
        creation_time = os.path.getctime(directory)
        creation_date = datetime.datetime.fromtimestamp(creation_time)

        # Count files and subdirectories
        file_count = 0
        dir_count = 0
        for _, dirs, files in os.walk(directory):
            file_count += len(files)
            dir_count += len(dirs)

        # Get directory name (last part of path without the prefix)
        dir_name = os.path.basename(directory)
        if dir_name.startswith("tempit_"):
            dir_name = dir_name[len("tempit_"):]
        else:
            dir_name = dir_name.split("_", 1)[-1]

        return {
            "path": directory,
            "name": dir_name,
            "size": total_size,
            "human_size": human_size,
            "created": creation_date,
            "file_count": file_count,
            "dir_count": dir_count
        }

    def _get_directory_name(self, directory: str, index: int) -> str:
        """ Get a user-friendly name for the directory based on its path and index """
        dir_name = os.path.basename(directory)

        # Check if it follows the pattern prefix_randomchars
        if '_' in dir_name:
            prefix = dir_name.split('_')[0]
            # If it's a tempit directory, append the index
            if prefix == "tempit":
                return f"{prefix}{index + 1}"
            # Otherwise just return the prefix
            return prefix

        # Fallback to the original basename if it doesn't match our pattern
        return dir_name

    def list_directories(self) -> List[str]:
        """ List all tracked temporary directories. """
        try:
            with open(self.tempit_file, "r") as file:
                directories = [line.strip() for line in file.readlines() if line.strip()]

            if not directories:
                return []

            # Validate that directories still exist
            existing_dirs = [d for d in directories if os.path.exists(d)]

            if len(existing_dirs) != len(directories):
                # Update file with only existing directories
                with open(self.tempit_file, "w") as file:
                    file.writelines(f"{d}\n" for d in existing_dirs)

            return existing_dirs

        except FileNotFoundError:
            return []

    def get_path_by_number(self, number: int) -> str:
        """ Get the path of a tracked temporary directory by its number """
        directories = self.list_directories()

        if not directories:
            return ""

        if not 1 <= number <= len(directories):
            print(colored(f"Invalid directory number: {number}", "red"))
            return ""

        return directories[number - 1]

    def _format_directory_data(self, dir_path: str, i: int) -> list:
        """Helper method to format directory data for display in tables"""
        info = self._get_directory_info(dir_path)
        created_str = info["created"].strftime("%Y-%m-%d %H:%M")

        # Get the friendly name
        friendly_name = self._get_directory_name(dir_path, i)
        name = colored(friendly_name, "cyan", attrs=["bold"])

        # Format contents
        contents_info = (
            f"{colored(info['file_count'], 'blue')} files, "
            f"{colored(info['dir_count'], 'blue')} dirs"
        )

        # Format size with appropriate color based on size
        size_color = "green"
        if info["size"] > 10 * 1024 * 1024:  # > 10MB
            size_color = "yellow"
        if info["size"] > 100 * 1024 * 1024:  # > 100MB
            size_color = "red"
        human_size = colored(info["human_size"], size_color)

        # Age calculation
        age = humanize.naturaltime(datetime.datetime.now() - info["created"])
        age_color = "green"
        if "day" in age or "month" in age or "year" in age:
            age_color = "yellow"
        colored_age = colored(age, age_color)

        return [
            colored(str(i + 1), "white", attrs=["bold"]),
            name,
            info["path"],
            human_size,
            created_str,
            colored_age,
            contents_info
        ]

    def _get_table_headers(self) -> list:
        """Return formatted table headers for directory listings"""
        return [
            colored("#", "white", attrs=["bold"]),
            colored("Name", "white", attrs=["bold"]),
            colored("Path", "white", attrs=["bold"]),
            colored("Size", "white", attrs=["bold"]),
            colored("Created", "white", attrs=["bold"]),
            colored("Age", "white", attrs=["bold"]),
            colored("Contents", "white", attrs=["bold"])
        ]

    def print_directories(self) -> None:
        """ Print a formatted table of tracked temporary directories. """
        try:
            directories = self.list_directories()

            if not directories:
                print(colored("No temporary directories found.", "yellow"))
                return

            # Create a nicely formatted table with enhanced information
            table_data = []
            for i, dir_path in enumerate(directories):
                row = self._format_directory_data(dir_path, i)
                table_data.append(row)

            print(colored("\n Temporary Directories", "green", attrs=["bold"]))
            headers = self._get_table_headers()

            # Use a more visually appealing table format
            print(tabulate(
                table_data,
                headers=headers,
                tablefmt="rounded_grid",
                numalign="center"
            ))
            print()  # Add a blank line after the table

        except FileNotFoundError:
            print(colored("No temporary directories tracking file found.", "yellow"))

    def remove(self, number: int) -> bool:
        """ Remove a tracked temporary directory by its number """
        directories = self.list_directories()

        if not directories:
            return False

        if not 1 <= number <= len(directories):
            print(colored(f"Invalid directory number: {number}", "red"))
            return False

        try:
            directory = directories[number - 1]
            shutil.rmtree(directory)

            # Update tracking file
            with open(self.tempit_file, "w") as file:
                file.writelines(f"{d}\n" for d in directories if d != directory)

            print(colored(f"Removed temporary directory: {directory}", "green"))
            return True

        except (IOError, OSError) as e:
            print(colored(f"Error removing temporary directory: {e}", "red"))
            return False

    def search_directories(self, search_str: str) -> None:
        """ Search for directories containing the specified string """
        directories = self.list_directories()

        if not directories:
            print(colored("No temporary directories found.", "yellow"))
            return

        matching_indices = [i for i, d in enumerate(directories) if search_str in d]

        if not matching_indices:
            print(colored(f"No directories found containing: {search_str}", "yellow"))
            return

        # Create a nicely formatted table with original indices and enhanced display
        table_data = []
        for i in matching_indices:
            dir_path = directories[i]
            row = self._format_directory_data(dir_path, i)
            table_data.append(row)

        print(colored(f"\n Directories containing '{search_str}'", "green", attrs=["bold"]))
        headers = self._get_table_headers()

        # Use a more visually appealing table format
        print(tabulate(
            table_data,
            headers=headers,
            tablefmt="rounded_grid",
            numalign="center"
        ))
        print()  # Add a blank line after the table

    def clean_all_directories(self) -> None:
        """ Remove all tracked temporary directories """
        directories = self.list_directories()

        if not directories:
            print(colored("No temporary directories found.", "yellow"))
            return

        for directory in directories:
            try:
                shutil.rmtree(directory)
            except (IOError, OSError) as e:
                print(colored(f"Error removing directory {directory}: {e}", "red"))

        # Clear the tracking file
        with open(self.tempit_file, "w") as _:
            pass

        print(colored("All temporary directories have been removed.", "green"))
