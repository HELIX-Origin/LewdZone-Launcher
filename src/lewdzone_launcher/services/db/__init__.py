"""DB access service — exposes typed queries over the Rule 06 connection.

Single writer connection; tokens are stored, never resolved URLs (ADR-0003).
"""

from __future__ import annotations

import sqlite3
from pathlib import Path

from lewdzone_launcher.services.db import core, migrations


def open(database_path: Path) -> sqlite3.Connection:  # noqa: A001
    """Open a connection and bring the schema to the latest migration."""
    conn = core.connect(database_path)
    core.migrate(conn, Path(migrations.__file__).parent)
    return conn