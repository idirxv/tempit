# pylint: disable=missing-module-docstring
# pylint: disable=redefined-outer-name
import os
from pathlib import Path
import shutil
import random
import humanize
import pytest
import tempit


@pytest.fixture
def manager(tmp_path):
    """ Create a TempitManager instance for testing. """
    tempdir_file = tmp_path / "tempit_dirs.txt"
    tempit_manager = tempit.TempitManager(str(tempdir_file))

    yield tempit_manager

    # Cleanup
    if os.path.exists(tmp_path):
        shutil.rmtree(tmp_path)


def _generate_random_files_and_subdirs(workdir: str) -> int:
    num_files = random.randint(1, 20)
    num_subdirs = random.randint(1, 5)
    num_files_per_subdir = random.randint(1, 20)

    file_size = 1024 * random.randint(1, 100)

    for _ in range(num_files):
        with open(f"{workdir}/file_{_}.txt", "wb") as f:
            f.write(os.urandom(file_size))

    for _ in range(num_subdirs):
        subdir = f"{workdir}/subdir_{_}"
        os.makedirs(subdir)
        for _ in range(num_files_per_subdir):
            with open(f"{subdir}/file_{_}.txt", "wb") as f:
                f.write(os.urandom(file_size))

    total_size = (num_files + num_files_per_subdir * num_subdirs) * file_size
    return total_size


def test_init_creates_file(manager):
    """ Check that the tracking file is created on initialization. """
    assert os.path.exists(manager.tempit_file), \
        "The tracking file should be created on initialization."


def test_create_directory(manager):
    """ Check that a directory can be created and listed. """
    new_dir = manager.create()
    assert os.path.isdir(new_dir), "The directory should be created on disk."

    # Check that the directory is listed
    dirs_list = manager.list_directories()
    assert new_dir in dirs_list, "The newly created directory should be listed."


def test_list_directories_cleanup(manager):
    """ Check that the list_directories method cleans up non-existing directories. """
    dir1 = manager.create()
    dir2 = manager.create()
    dir3 = manager.create()

    # Remove the second directory
    shutil.rmtree(dir2)

    dirs_list = manager.list_directories()
    # Check that the second directory is not listed
    assert dir1 in dirs_list, "The existing directories should still be listed."
    assert dir2 not in dirs_list, "The removed directory should not be listed."
    assert dir3 in dirs_list, "The existing directories should still be listed."


def test_remove_valid_directory(manager):
    """ Check that a directory can be removed. """
    _ = manager.create()        # directory 1
    dir2 = manager.create()     # directory 2
    _ = manager.create()        # directory 3

    # Remove the second directory
    success = manager.remove(2)
    assert success, "The removal should succeed for a valid index."

    dirs_list = manager.list_directories()
    assert dir2 not in dirs_list, "The removed directory should not be listed."
    assert not os.path.exists(dir2), "The directory should be removed from disk."


def test_remove_invalid_directory(manager, capsys):
    """ Check that an error message is displayed for an invalid directory index. """
    dir1 = manager.create()  # directory 1

    # Try to remove a directory with an invalid index
    success = manager.remove(2)
    captured = capsys.readouterr()

    assert not success, "The removal should fail for an invalid index."
    assert "Invalid directory number" in captured.out, (
        "An error message should be displayed for an invalid index."
    )

    # Check that the directory is still listed
    dirs_list = manager.list_directories()
    assert dir1 in dirs_list, "The directory should still be listed."


def test_print_directories_no_directories(manager, capsys):
    """ Check that a message is displayed when no directories are found. """
    manager.print_directories()
    captured = capsys.readouterr()
    assert "No temporary directories found" in captured.out, (
        "A message should be displayed when no directories are found."
    )


def test_print_directories_with_content(manager, capsys):
    """ Check that the directories are displayed in a table. """
    dir1 = manager.create()
    dir2 = manager.create()

    manager.print_directories()
    captured = capsys.readouterr()

    # Vérifions que les deux répertoires sont affichés
    assert dir1 in captured.out, "Le répertoire créé devrait être affiché."
    assert dir2 in captured.out, "Le répertoire créé devrait être affiché."
    assert "Temporary Directories" in captured.out, "Le titre du tableau devrait être affiché."


def test_remove_directory_with_content(manager):
    """ Check that a directory with content can be removed. """
    manager.create()
    dir2 = manager.create()

    file1 = Path(dir2) / "file1.txt"
    file2 = Path(dir2) / "file2.txt"

    # Generate random files with random content
    with open(file1, "wb") as f1, open(file2, "wb") as f2:
        f1.write(os.urandom(1024 * 1024))
        f2.write(os.urandom(1024 * 1024))

    # Remove the second directory
    success = manager.remove(2)
    assert success, "The removal should succeed for a valid index."
    assert not os.path.exists(dir2), "The directory should be removed from disk."


def test_create_directory_custom_prefix(manager):
    """ Check that a directory can be created with a custom prefix. """
    prefix = "my_temp_dir"
    new_dir = manager.create(prefix=prefix)
    assert new_dir.startswith(f"{manager.tempit_dir}/{prefix}"), \
        "The directory should be created with the custom prefix."

    # Check that the directory is listed
    dirs_list = manager.list_directories()
    assert new_dir in dirs_list, "The newly created directory should be listed."


def test_directory_size_from_list(manager, capsys):
    """ Check that the size of the directories is displayed. """
    dir1 = manager.create()
    dir2 = manager.create()

    real_total_size1 = _generate_random_files_and_subdirs(dir1)
    real_total_size2 = _generate_random_files_and_subdirs(dir2)

    manager.print_directories()
    captured = capsys.readouterr()

    # Check that the total size of the directories is displayed
    assert humanize.naturalsize(real_total_size1, binary=True) in captured.out, \
        "The total size of the first directory should be displayed."
    assert humanize.naturalsize(real_total_size2, binary=True) in captured.out, \
        "The total size of the second directory should be displayed."
