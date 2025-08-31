import sys

import tempit.cli as cli


def test_cli_create(monkeypatch, capsys, tmp_path):
    created = tmp_path / "tempit_created"

    class DummyManager:
        def create(self, prefix):
            return str(created)

    dummy = DummyManager()
    monkeypatch.setattr(cli, "TempitManager", lambda: dummy)
    monkeypatch.setattr(sys, "argv", ["tempit", "--create", "custom"])

    cli.main()
    captured = capsys.readouterr()
    assert str(created) in captured.out


def test_cli_list(monkeypatch, capsys):
    class DummyManager:
        def print_directories(self):
            print("listed")

    monkeypatch.setattr(cli, "TempitManager", lambda: DummyManager())
    monkeypatch.setattr(sys, "argv", ["tempit", "--list"])

    cli.main()
    captured = capsys.readouterr()
    assert "listed" in captured.out


def test_cli_get(monkeypatch, capsys):
    class DummyManager:
        def get_path_by_number(self, number):
            return f"/tmp/path{number}"

    monkeypatch.setattr(cli, "TempitManager", lambda: DummyManager())
    monkeypatch.setattr(sys, "argv", ["tempit", "--get", "2"])

    cli.main()
    captured = capsys.readouterr()
    assert "/tmp/path2" in captured.out


def test_cli_remove(monkeypatch):
    calls = {}

    class DummyManager:
        def remove(self, number):
            calls["number"] = number

    monkeypatch.setattr(cli, "TempitManager", lambda: DummyManager())
    monkeypatch.setattr(sys, "argv", ["tempit", "--remove", "3"])

    cli.main()
    assert calls["number"] == 3


def test_cli_search(monkeypatch, capsys):
    class DummyManager:
        def search_directories(self, term):
            print(f"search:{term}")

    monkeypatch.setattr(cli, "TempitManager", lambda: DummyManager())
    monkeypatch.setattr(sys, "argv", ["tempit", "--search", "foo"])

    cli.main()
    captured = capsys.readouterr()
    assert "search:foo" in captured.out


def test_cli_clean_all(monkeypatch):
    called = []

    class DummyManager:
        def clean_all_directories(self):
            called.append(True)

    monkeypatch.setattr(cli, "TempitManager", lambda: DummyManager())
    monkeypatch.setattr(sys, "argv", ["tempit", "--clean-all"])

    cli.main()
    assert called


def test_cli_init(monkeypatch, capsys):
    class DummyManager:
        def init_shell(self, shell):
            print(f"init:{shell}")

    monkeypatch.setattr(cli, "TempitManager", lambda: DummyManager())
    monkeypatch.setattr(sys, "argv", ["tempit", "--init", "bash"])

    cli.main()
    captured = capsys.readouterr()
    assert "init:bash" in captured.out
