"""CLI module for the tempit application."""

from argparse import ArgumentParser, SUPPRESS
import logging
import sys

from tempit.core import TempitManager


def main() -> int:
    """Main CLI entry point for the Tempit application."""
    # Set up basic logging
    logging.basicConfig(level=logging.WARNING, format="%(levelname)s: %(message)s")

    parser = ArgumentParser(description="Manage temporary directories.")
    parser.add_argument("-c", "--create", nargs="?", const="tempit",
                        help="Create a new temporary directory.")
    parser.add_argument("-init", "--init", type=str,
                        help="Initialize Tempit in the current shell.")
    parser.add_argument("-l", "--list", action="store_true",
                        help="List all tracked temporary directories.")
    parser.add_argument("-rm", "--remove", type=int,
                        help="Remove a tracked temporary directory by its number.")
    parser.add_argument("--clean-all", action="store_true",
                        help="Remove all tracked temporary directories.")
    parser.add_argument("-p", "--path", type=int, help=SUPPRESS)
    args = parser.parse_args()

    try:
        manager = TempitManager()

        if args.init:
            manager.init_shell(args.init)
        elif args.create:
            print(manager.create(args.create))
        elif args.list:
            manager.print_directories()
        elif args.remove:
            success = manager.remove(args.remove)
            if not success:
                return 1
        elif args.clean_all:
            manager.clean_all_directories()
        elif args.path:
            print(manager.storage.get_path_by_number(args.path))
        else:
            parser.print_help()
    except (IOError, OSError) as e:
        logging.error("An error occurred: %s", e)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
