# pylint: disable=missing-module-docstring,redefined-outer-name
from pathlib import Path

import pytest

from tempit.core import TempitManager


@pytest.fixture
def tempit_manager(tmp_path):
    """Create a TempitManager instance with a temporary tracking file"""
    return TempitManager(storage_file=Path(tmp_path) / "tempit_dirs.json")


def test_init_shell(tempit_manager, capsys):
    """Test printing init script"""
    tempit_manager.init_shell("bash")
    captured = capsys.readouterr()
    assert "TEMPIT_EXE" in captured.out


def test_create_and_list_directories(tempit_manager):
    """Test creating and listing directories"""
    file_path = tempit_manager.create()
    assert file_path.is_dir()
    assert file_path == tempit_manager.storage.get_all_directories()[0].path


def test_get_path_by_number(tempit_manager):
    """Test getting path by number"""
    path1 = tempit_manager.create(prefix="first")
    path2 = tempit_manager.create(prefix="second")
    assert path1 == tempit_manager.storage.get_path_by_number(1)
    assert path2 == tempit_manager.storage.get_path_by_number(2)


def test_remove_directory(tempit_manager):
    """Test removing directory"""
    path1 = tempit_manager.create(prefix="first")
    path2 = tempit_manager.create(prefix="second")
    assert tempit_manager.remove(1) is True
    assert not path1.exists()
    assert path2 == tempit_manager.storage.get_all_directories()[0].path


def test_clean_all_directories(tempit_manager):
    """Test cleaning all directories"""
    path1 = tempit_manager.create(prefix="first")
    path2 = tempit_manager.create(prefix="second")
    tempit_manager.clean_all_directories()
    assert not path1.exists()
    assert not path2.exists()
    assert [] == tempit_manager.storage.get_all_directories()


def test_get_all_directories_is_pure_read(tempit_manager, tmp_path):
    """get_all_directories() should not remove stale entries from storage."""
    # Create a dir, record it, then delete it from filesystem
    path = tempit_manager.create(prefix="stale")
    path.rmdir()
    # Raw storage should still have the entry
    all_dirs = tempit_manager.storage.get_all_directories()
    assert any(d.path == path for d in all_dirs)


def test_prune_stale_removes_missing_entries(tempit_manager):
    """prune_stale() should remove entries whose paths no longer exist."""
    path = tempit_manager.create(prefix="stale")
    path.rmdir()
    tempit_manager.storage.prune_stale()
    all_dirs = tempit_manager.storage.get_all_directories()
    assert not any(d.path == path for d in all_dirs)
