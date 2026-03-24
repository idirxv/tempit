import pytest
from tempit.models import DirectoryInfo, DirectoryStats
from tempit.stats import calculate_stats
from datetime import datetime


@pytest.fixture
def dir_info(tmp_path):
    return DirectoryInfo(path=tmp_path, created=datetime.now(), prefix="test")


def test_calculate_stats_returns_stats_for_existing_dir(dir_info):
    stats = calculate_stats(dir_info)
    assert stats is not None
    assert isinstance(stats, DirectoryStats)
    assert stats.size_bytes >= 0
    assert stats.file_count >= 0
    assert stats.dir_count >= 0
    assert stats.human_size != ""
    assert stats.age != ""


def test_calculate_stats_returns_none_for_missing_dir(tmp_path):
    missing = DirectoryInfo(path=tmp_path / "nonexistent", created=datetime.now(), prefix="test")
    assert calculate_stats(missing) is None


def test_calculate_stats_counts_files(dir_info):
    (dir_info.path / "file.txt").write_text("hello")
    stats = calculate_stats(dir_info)
    assert stats.file_count == 1
    assert stats.size_bytes == 5
