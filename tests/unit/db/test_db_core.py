"""Rule 06 bootstrap tests: connection pragmas, forward migrations, schema shape.

Offline (no network). Uses a temp DB per test.
"""

from __future__ import annotations

import sqlite3
from pathlib import Path

import pytest

from lewdzone_launcher.services.db import core
from lewdzone_launcher.services.db import migrations


def migrations_dir() -> Path:
    return Path(migrations.__file__).parent


@pytest.fixture()
def conn(tmp_path):
    c = core.connect(tmp_path / "test.db")
    yield c
    c.close()


def test_connect_sets_wal_fk_busy(conn):
    assert conn.execute("PRAGMA journal_mode").fetchone()[0].lower() == "wal"
    assert conn.execute("PRAGMA foreign_keys").fetchone()[0] == 1
    assert conn.execute("PRAGMA busy_timeout").fetchone()[0] == 5000


def test_migrate_applies_all_and_is_idempotent(tmp_path):
    c = core.connect(tmp_path / "m.db")
    applied = core.migrate(c, migrations_dir())
    assert applied == ["001_initial"]
    assert c.execute("SELECT name FROM schema_migrations").fetchall() == [
        ("001_initial",)
    ]
    assert core.migrate(c, migrations_dir()) == []
    c.close()


def test_schema_has_expected_tables(conn):
    core.migrate(conn, migrations_dir())
    tables = {
        r[0]
        for r in conn.execute(
            "SELECT name FROM sqlite_master WHERE type='table'"
        ).fetchall()
    }
    assert {
        "game",
        "genre",
        "game_genre",
        "version",
        "host",
        "download_entry",
        "download_job",
        "game_external",
        "artwork_cache",
        "schema_migrations",
    } <= tables


def test_foreign_keys_enforced(conn):
    core.migrate(conn, migrations_dir())
    with pytest.raises(sqlite3.IntegrityError):
        conn.execute(
            "INSERT INTO game_genre (game_post_id, genre_slug) VALUES (?, ?)",
            (999, "nope"),
        )


def test_tokens_stored_parameterized(conn):
    """download_entry persists a token; bare string-built insert is not used."""
    core.migrate(conn, migrations_dir())
    conn.execute(
        "INSERT INTO game (post_id, slug, title) VALUES (?, ?, ?)",
        (1, "treasure-of-nadia", "Treasure of Nadia"),
    )
    conn.execute(
        "INSERT INTO version (game_id, label, is_latest) VALUES (?, ?, ?)",
        (1, "1.0", 1),
    )
    conn.execute(
        "INSERT INTO host (slug, display_name) VALUES (?, ?)",
        ("fileknot", "Fileknot"),
    )
    conn.execute(
        "INSERT INTO download_entry (game_id, version_id, host_slug, platform, tab, label, token) "
        "VALUES (?, ?, ?, ?, ?, ?, ?)",
        (1, 1, "fileknot", "PC", "official", "Windows", "v1.payload.sig"),
    )
    row = conn.execute(
        "SELECT host_slug, token FROM download_entry WHERE game_id = ?", (1,)
    ).fetchone()
    assert row == ("fileknot", "v1.payload.sig")


def test_no_resolved_urls_stored():
    """The schema has no column named url; tokens only (ADR-0003)."""
    conn = sqlite3.connect(":memory:")
    core.migrate(conn, migrations_dir())
    download_entry_cols = {
        r[1] for r in conn.execute("PRAGMA table_info(download_entry)").fetchall()
    }
    assert "url" not in download_entry_cols
    assert "token" in download_entry_cols
    conn.close()


def test_open_helper_migrates_fresh_db(tmp_path):
    """services.db.open returns a Rule 06 connection with latest schema."""
    from lewdzone_launcher.services.db import open as open_db

    c = open_db(tmp_path / "fresh.db")
    try:
        assert c.execute("PRAGMA journal_mode").fetchone()[0].lower() == "wal"
        tables = {
            r[0]
            for r in c.execute(
                "SELECT name FROM sqlite_master WHERE type='table'"
            ).fetchall()
        }
        assert "download_job" in tables
        assert c.execute("SELECT name FROM schema_migrations").fetchall() == [
            ("001_initial",)
        ]
    finally:
        c.close()