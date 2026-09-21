"""Shared fixtures for the repo-hygiene scanner suite (scanners)."""

from __future__ import annotations

import pathlib
import re

import pytest

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent

# Paths that must never be scanned (build/packaging/junk).
SKIP_DIRS = {
    ".git",
    ".venv",
    ".pytest_cache",
    "__pycache__",
    "scratch",          # intentional temp scripts
    "build",            # installer output (gitignored)
    "node_modules",
    "dist",
    "target",
    "gen",
    ".ruff_cache",
    ".mypy_cache",
    ".pyright",
}


def skipped(path: pathlib.Path) -> bool:
    return any(part in SKIP_DIRS for part in path.parts)


# Scanner test modules define their own needle literals; never scan them.
def _in_scanners(path: pathlib.Path) -> bool:
    return "unit" in path.parts and "scanners" in path.parts


def iter_repo_files(suffixes: tuple[str, ...] | None = None) -> list[pathlib.Path]:
    """Return all tracked-style repo files, skipping ignored/build/scanner dirs."""
    suffixes = suffixes or (".md", ".py", ".toml", ".yml", ".yaml", ".json")
    return sorted(
        p
        for p in REPO_ROOT.rglob("*")
        if p.is_file() and p.suffix in suffixes and not skipped(p) and not _in_scanners(p)
    )


@pytest.fixture(scope="session")
def repo_root() -> pathlib.Path:
    return REPO_ROOT


@pytest.fixture(scope="session")
def repo_md_files() -> list[pathlib.Path]:
    return [p for p in iter_repo_files() if p.suffix == ".md"]


@pytest.fixture(scope="session")
def repo_py_files() -> list[pathlib.Path]:
    return [p for p in iter_repo_files() if p.suffix == ".py"]


# cp1252-misread UTF-8 double-encoding survivors (from scratch/scan_mojibake.py)
MOJIBAKE_RE = re.compile(
    r"[\u00c3\u00e2\u00f0\u0178\u00ef\u00b8\u00c5\u00a1\u2039\u00ba\u00bb"
    r"\u017e\u0152\u017d\u0161\u2030\u02c6\u2018\u2019]"
)