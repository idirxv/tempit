# pylint: disable=missing-module-docstring,redefined-outer-name
from pathlib import Path

import pytest
from typer.testing import CliRunner

from tempit import cli
from tempit.core import TempitManager

runner = CliRunner()


@pytest.fixture
def create_manager(tmp_path):
    """Create a TempitManager instance with a temporary tracking file"""
    return TempitManager(storage_file=Path(tmp_path) / "tempit_dirs.json")


def test_cli_init():
    """Test printing init script"""
    result = runner.invoke(cli.app, ["init", "bash"])
    assert result.exit_code == 0
    assert "TEMPIT_EXE" in result.output


def test_cli_create(create_manager, monkeypatch):
    """Test creating directories via CLI"""
    monkeypatch.setattr(cli, "get_manager", lambda: create_manager)

    runner.invoke(cli.app, ["create", "custom_prefix"])
    runner.invoke(cli.app, ["create", "tempit"])

    directories = create_manager.storage.get_all_directories()
    assert len(directories) == 2
    assert "custom_prefix" == directories[0].prefix
    assert "tempit" == directories[1].prefix


def test_cli_list(create_manager, monkeypatch):
    """Test listing directories via CLI"""
    monkeypatch.setattr(cli, "get_manager", lambda: create_manager)

    create_manager.create("custom_prefix1")
    create_manager.create("custom_prefix2")

    result = runner.invoke(cli.app, ["list"])
    assert result.exit_code == 0
    assert "custom_prefix1" in result.output
    assert "custom_prefix2" in result.output


def test_cli_remove(create_manager, monkeypatch):
    """Test removing directories via CLI"""
    monkeypatch.setattr(cli, "get_manager", lambda: create_manager)

    create_manager.create("custom_prefix1")
    create_manager.create("custom_prefix2")

    result = runner.invoke(cli.app, ["remove", "1"])
    assert result.exit_code == 0

    list_dir = create_manager.storage.get_all_directories()
    assert len(list_dir) == 1
    assert "custom_prefix1" not in list_dir[0].prefix
    assert "custom_prefix2" in list_dir[0].prefix


def test_cli_clean_all(create_manager, monkeypatch):
    """Test cleaning all directories via CLI"""
    monkeypatch.setattr(cli, "get_manager", lambda: create_manager)

    create_manager.create("custom_prefix1")
    create_manager.create("custom_prefix2")

    result = runner.invoke(cli.app, ["clean-all"])
    assert result.exit_code == 0

    list_dir = create_manager.storage.get_all_directories()
    assert len(list_dir) == 0
