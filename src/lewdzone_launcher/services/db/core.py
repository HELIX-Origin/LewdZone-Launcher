"""SQLite connection setup and schema migrations.

Implements Rule 06: WAL + foreign_keys + busy_timeout, forward-only migrations
tracked in a ``schema_migrations`` table, parameterized SQL only.
"""

from __future__ import annotations

import sqlite3
from importlib import import_module
from pathlib import Path

from lewdzone_launcher.domain.exceptions import LewdzoneError


def connect(database_path: Path) -> sqlite3.Connection:
    """Open a Rule 06-compliant connection (WAL, FK, busy_timeout)."""
    conn = sqlite3.connect(str(database_path))
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA foreign_keys=ON")
    conn.execute("PRAGMA busy_timeout=5000")
    return conn


def migrate(conn: sqlite3.Connection, migrations_dir: Path) -> list[str]:
    """Apply forward-only migrations not yet recorded in schema_migrations.

    Migration modules are named ``NNN_desc.py`` and expose ``upgrade(conn)``.
    Returns the list of applied ``NNN_desc`` names in order.
    """
    _ensure_schema_migrations(conn)
    applied = {
        row[0]
        for row in conn.execute("SELECT name FROM schema_migrations").fetchall()
    }
    names = sorted(p.stem for p in migrations_dir.glob("[0-9][0-9][0-9]_*.py"))
    newly_applied: list[str] = []
    for name in names:
        if name in applied:
            continue
        module = import_module(f"lewdzone_launcher.services.db.migrations.{name}")
        with conn:
            module.upgrade(conn)
            conn.execute(
                "INSERT INTO schema_migrations (name) VALUES (?)",
                (name,),
            )
        newly_applied.append(name)
    return newly_applied


def _ensure_schema_migrations(conn: sqlite3.Connection) -> None:
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations ("
        " name TEXT PRIMARY KEY,"
        " applied_at TEXT NOT NULL DEFAULT (datetime('now'))"
        ")"
    )


class DbError(LewdzoneError):
    """Unexpected database state/operation failure (exit code 1)."""