"""Scanner: reject UTF-8 double-encoding mojibake in repo files.

Windows tooling (PowerShell 5.1 ``Set-Content``) can misread UTF-8 as cp1252
and re-encode, producing the corruption classes in ``MOJIBAKE_RE``. These
tests are the always-on guard that keeps such bytes out (port of
``scratch/scan_mojibake.py``).
"""

from __future__ import annotations

import pytest

from tests.conftest import MOJIBAKE_RE, REPO_ROOT, iter_repo_files


def _mojibake_lines(path):
    bad = []
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return []
    for lineno, line in enumerate(text.splitlines(), 1):
        if MOJIBAKE_RE.search(line):
            bad.append((lineno, line))
    return bad


@pytest.mark.unit
@pytest.mark.parametrize(
    "path",
    iter_repo_files(),
    ids=lambda p: str(p.relative_to(REPO_ROOT)),
)
def test_no_mojibake_in_file(path):
    """Every repo file is free of cp1252-mojibake marker characters."""
    lines = _mojibake_lines(path)
    assert not lines, f"{path}: mojibake at line(s) {[n for n, _ in lines]}"


@pytest.mark.unit
def test_repo_has_markdown_to_screen():
    """Standalone probe so the suite can never silently shrink to zero files."""
    assert len(iter_repo_files()) > 50