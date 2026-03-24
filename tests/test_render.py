import sys
from datetime import datetime
from pathlib import Path

import pytest
from tempit.models import DirectoryInfo, DirectoryStats
from tempit.render import DirectoryRenderer


@pytest.fixture
def renderer():
    return DirectoryRenderer()


@pytest.fixture
def sample_entries(tmp_path):
    info = DirectoryInfo(path=tmp_path, created=datetime.now(), prefix="test")
    stats = DirectoryStats(
        size_bytes=1024,
        human_size="1.0 KiB",
        file_count=2,
        dir_count=1,
        age="just now",
    )
    return [(info, stats)]


def test_renderer_requires_no_constructor_args():
    """Renderer should be constructable with no arguments."""
    renderer = DirectoryRenderer()
    assert renderer is not None


def test_render_empty_list_prints_message(renderer, capsys):
    renderer.render_directory_list([])
    captured = capsys.readouterr()
    assert "No temporary directories found" in captured.out


def test_render_entries_shows_path(renderer, sample_entries, capsys):
    renderer.render_directory_list(sample_entries)
    captured = capsys.readouterr()
    path_str = str(sample_entries[0][0].path)
    assert path_str in captured.out
