"""Per-OS configuration and data paths.

| OS      | Config dir                        | Data dir                        |
| ---     | ---                               | ---                             |
| Windows | ``%APPDATA%\\lewdzone``           | ``%LOCALAPPDATA%\\lewdzone``    |
| Linux   | ``$XDG_CONFIG_HOME`` or ``~/.config`` | ``$XDG_DATA_HOME`` or ``~/.local/share`` |
| macOS   | ``~/Library/Application Support/lewdzone`` | same per system |
"""

from __future__ import annotations

import os
import sys
from pathlib import Path

APP_DIR_NAME = "lewdzone"


def _os_name() -> str:
    return sys.platform


def config_dir() -> Path:
    """Return the per-OS configuration directory, creating it if needed."""
    platform = _os_name()
    if platform == "win32":
        base = Path(os.environ.get("APPDATA", Path.home() / "AppData" / "Roaming"))
    elif platform == "darwin":
        base = Path.home() / "Library" / "Application Support"
    else:
        base = Path(os.environ.get("XDG_CONFIG_HOME", Path.home() / ".config"))
    return _ensure(base / APP_DIR_NAME)


def data_dir() -> Path:
    """Return the per-OS data directory (database, cache), creating it."""
    platform = _os_name()
    if platform == "win32":
        base = Path(os.environ.get("LOCALAPPDATA", Path.home() / "AppData" / "Local"))
    elif platform == "darwin":
        base = Path.home() / "Library" / "Application Support"
    else:
        base = Path(os.environ.get("XDG_DATA_HOME", Path.home() / ".local" / "share"))
    return _ensure(base / APP_DIR_NAME)


def database_path() -> Path:
    """Return ``<data_dir>/lewdzone.db``."""
    return data_dir() / "lewdzone.db"


def _ensure(path: Path) -> Path:
    path.mkdir(parents=True, exist_ok=True)
    return path