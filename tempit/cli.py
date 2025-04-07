#!/usr/bin/env python3

# pylint: disable=missing-module-docstring
import argparse
import sys
from .core import TempitManager


def main():
    """ Main CLI entry point for the Tempit application """
    parser = argparse.ArgumentParser(description="Manage temporary directories.")
    parser.add_argument("-c", "--create", nargs="?", const="tempit",
                        help="Create a new temporary directory.")
    parser.add_argument("-l", "--list", action="store_true",
                        help="List all tracked temporary directories.")
    parser.add_argument("-g", "--get", type=int,
                        help="Get the path of a tracked temporary directory by its number.")
    parser.add_argument("-r", "--remove", type=int,
                        help="Remove a tracked temporary directory by its number.")
    parser.add_argument("-s", "--search", type=str,
                        help="Search for directories containing the specified string.")
    parser.add_argument("--clean-all", action="store_true",
                        help="Remove all tracked temporary directories.")
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
    elif args.search:
        manager.search_directories(args.search)
    elif args.clean_all:
        manager.clean_all_directories()
    else:
        parser.print_help()


if __name__ == "__main__":
    sys.exit(main())
