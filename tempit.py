# pylint: disable=missing-module-docstring
# pylint: disable=unspecified-encoding
import os
import tempfile
import shutil
import argparse
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

    def print_directories(self) -> None:
        """ Print a formatted table of tracked temporary directories. """
        try:
            directories = self.list_directories()

            if not directories:
                print(colored("No temporary directories found.", "yellow"))
                return

            # Create a nicely formatted table
            table_data = [
                [i + 1, dir, self._get_directory_size(dir)[1]]
                for i, dir in enumerate(directories)
            ]
            print(colored("Temporary Directories", "green", attrs=["bold"]))
            print(tabulate(
                table_data,
                headers=["#", "Directory Path", "Size"],
                tablefmt="fancy_grid",
                numalign="center"
            ))

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


def main():
    """ Main function """
    parser = argparse.ArgumentParser(description="Manage temporary directories.")
    parser.add_argument("-c", "--create", nargs="?", const="tempit",
                        help="Create a new temporary directory.")
    parser.add_argument("-l", "--list", action="store_true",
                        help="List all tracked temporary directories.")
    parser.add_argument("-g", "--get", type=int,
                        help="Get the path of a tracked temporary directory by its number.")
    parser.add_argument("-r", "--remove", type=int,
                        help="Remove a tracked temporary directory by its number.")
    args = parser.parse_args()

    manager = TempitManager()

    if args.create:
        manager.create(args.create)
    elif args.list:
        manager.print_directories()
    elif args.get:
        print(manager.get_path_by_number(args.get))
    elif args.remove:
        manager.remove(args.remove)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
