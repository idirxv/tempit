# pylint: disable=missing-module-docstring
import os

import pytest

from tempit.core import TempitManager


@pytest.fixture
def create_manager(tmp_path):
    """Create a TempitManager instance with a temporary tracking file"""
    return TempitManager(tempit_file=str(tmp_path / "tempit_dirs.txt"))


def test_create_and_list_directories(create_manager):
    """Test creating and listing directories"""
    path = create_manager.create()
    assert os.path.isdir(path)
    assert create_manager.list_directories() == [path]


def test_get_path_by_number(create_manager):
    """Test getting path by number"""
    path1 = create_manager.create(prefix="first")
    path2 = create_manager.create(prefix="second")
    assert create_manager.get_path_by_number(1) == path1
    assert create_manager.get_path_by_number(2) == path2


def test_remove_directory(create_manager):
    """Test removing directory"""
    path1 = create_manager.create(prefix="first")
    path2 = create_manager.create(prefix="second")
    assert create_manager.remove(1) is True
    assert not os.path.exists(path1)
    assert create_manager.list_directories() == [path2]


def test_clean_all_directories(create_manager):
    """Test cleaning all directories"""
    path1 = create_manager.create(prefix="first")
    path2 = create_manager.create(prefix="second")
    create_manager.clean_all_directories()
    assert not os.path.exists(path1)
    assert not os.path.exists(path2)
    assert create_manager.list_directories() == []
