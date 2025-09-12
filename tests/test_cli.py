# pylint: disable=missing-module-docstring,redefined-outer-name
import sys
from pathlib import Path

import pytest

from tempit import cli
from tempit.core import TempitManager


@pytest.fixture
def create_manager(tmp_path):
    """Create a TempitManager instance with a temporary tracking file"""
    return TempitManager(storage_file=Path(tmp_path) / "tempit_dirs.json")


def test_cli_init(monkeypatch, capsys):
    """Test printing init script"""
    monkeypatch.setattr(sys, "argv", ["tempit", "--init", "bash"])
    cli.main()

    captured = capsys.readouterr()

    assert "TEMPIT_EXE" in captured.out


def test_cli_create(create_manager, monkeypatch):
    """Test creating directories via CLI"""
    monkeypatch.setattr(cli, "TempitManager", lambda: create_manager)

    # Create a directory via CLI
    monkeypatch.setattr(sys, "argv", ["tempit", "--create", "custom_prefix"])
    cli.main()

    monkeypatch.setattr(sys, "argv", ["tempit", "--create"])
    cli.main()

    # Check if the directory is created
    directories = create_manager.storage.get_existing_directories()
    assert len(directories) == 2
    assert "custom_prefix" == directories[0].prefix
    assert "tempit" == directories[1].prefix


def test_cli_list(create_manager, monkeypatch, capsys):
    """Test listing directories via CLI"""
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


def test_cli_remove(create_manager, monkeypatch):
    """Test removing directories via CLI"""
    monkeypatch.setattr(cli, "TempitManager", lambda: create_manager)

    # Create directories
    create_manager.create("custom_prefix1")
    create_manager.create("custom_prefix2")

    # Remove directory via CLI
    monkeypatch.setattr(sys, "argv", ["tempit", "--remove", "1"])
    cli.main()

    # List directories via CLI
    list_dir = create_manager.storage.get_existing_directories()

    assert len(list_dir) == 1
    assert "custom_prefix1" not in list_dir[0].prefix
    assert "custom_prefix2" in list_dir[0].prefix


def test_cli_clean_all(create_manager, monkeypatch):
    """Test cleaning all directories via CLI"""
    monkeypatch.setattr(cli, "TempitManager", lambda: create_manager)

    # Create directories
    create_manager.create("custom_prefix1")
    create_manager.create("custom_prefix2")

    # Clean all directories via CLI
    monkeypatch.setattr(sys, "argv", ["tempit", "--clean-all"])
    cli.main()

    # List directories via CLI
    list_dir = create_manager.storage.get_existing_directories()
    assert len(list_dir) == 0
