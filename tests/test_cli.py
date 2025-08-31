# pylint: disable=missing-module-docstring
import sys

import pytest

from tempit import cli
from tempit.core import TempitManager


@pytest.fixture
def create_manager(tmp_path):
    """Create a TempitManager instance with a temporary tracking file"""
    return TempitManager(tempit_file=str(tmp_path / "tempit_dirs.txt"))


def test_cli_create(create_manager, monkeypatch):
    monkeypatch.setattr(cli, "TempitManager", lambda: create_manager)

    # Create a directory via CLI
    monkeypatch.setattr(sys, "argv", ["tempit", "--create", "custom_prefix"])
    cli.main()

    # Check if the directory is created
    list_dir = create_manager.list_directories()
    assert len(list_dir) == 1
    assert "custom_prefix" in list_dir[0]


def test_cli_list(create_manager, monkeypatch, capsys):
    monkeypatch.setattr(cli, "TempitManager", lambda: create_manager)

    # Create a directory
    create_manager.create("custom_prefix1")
    create_manager.create("custom_prefix2")

    # List directories via CLI
    monkeypatch.setattr(sys, "argv", ["tempit", "--list"])
    cli.main()

    captured = capsys.readouterr()
    assert "custom_prefix1" in captured.out
    assert "custom_prefix2" in captured.out


def test_cli_get(create_manager, monkeypatch, capsys):
    monkeypatch.setattr(cli, "TempitManager", lambda: create_manager)

    # Create directories
    create_manager.create("custom_prefix1")
    create_manager.create("custom_prefix2")

    # Get directory via CLI
    monkeypatch.setattr(sys, "argv", ["tempit", "--get", "2"])
    cli.main()

    captured = capsys.readouterr()
    assert "custom_prefix2" in captured.out


def test_cli_remove(create_manager, monkeypatch):
    monkeypatch.setattr(cli, "TempitManager", lambda: create_manager)

    # Create directories
    create_manager.create("custom_prefix1")
    create_manager.create("custom_prefix2")

    # Remove directory via CLI
    monkeypatch.setattr(sys, "argv", ["tempit", "--remove", "1"])
    cli.main()

    # List directories via CLI
    list_dir = create_manager.list_directories()
    assert len(list_dir) == 1
    assert "custom_prefix2" in list_dir[0]


def test_cli_search(create_manager, monkeypatch, capsys):
    monkeypatch.setattr(cli, "TempitManager", lambda: create_manager)

    # Create directories
    create_manager.create("custom_prefix1")
    create_manager.create("custom_prefix2")

    # Search directories via CLI
    monkeypatch.setattr(sys, "argv", ["tempit", "--search", "custom"])
    cli.main()

    captured = capsys.readouterr()
    
    assert "custom_prefix1" in captured.out
    assert "custom_prefix2" in captured.out


def test_cli_clean_all(create_manager, monkeypatch):
    monkeypatch.setattr(cli, "TempitManager", lambda: create_manager)

    # Create directories
    create_manager.create("custom_prefix1")
    create_manager.create("custom_prefix2")

    # Clean all directories via CLI
    monkeypatch.setattr(sys, "argv", ["tempit", "--clean-all"])
    cli.main()

    # List directories via CLI
    list_dir = create_manager.list_directories()
    assert len(list_dir) == 0


def test_cli_init(monkeypatch, capsys):

    monkeypatch.setattr(sys, "argv", ["tempit", "--init", "bash"])
    cli.main()

    captured = capsys.readouterr()
    
    assert "TEMPIT_EXE" in captured.out
