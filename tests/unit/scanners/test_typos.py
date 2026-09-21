"""Scanner: reject typos in repo files via codespell.

Runs ``codespell -H`` over all repo files (hidden dirs included, so
``.agents`` and ``wiki`` are screened). This is the always-on guard for the
"no typos in documentation or roadmap" requirement.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

from tests.conftest import REPO_ROOT, SKIP_DIRS

# --ignore-words-list: project vocabulary codespell does not know.
_IGNORE = [
    "lewdzone",      # project name
    "renpy",         # Ren'Py engine
    "npcs",          # plural NPC
    "kanji",         # game content term
    "hentai",        # genre label
    "yuri",          # genre label
    "yaoi",          # genre label
]
_SKIP = ",".join(
    [".git", ".venv", "scratch", "build", "node_modules", "dist", "target", "gen"]
)


def _codespell_check() -> tuple[int, str]:
    proc = subprocess.run(
        [
            sys.executable,
            "-m",
            "codespell_lib",           # codespell's importable module
            "-H",                      # include hidden directories
            "-L",
            ",".join(_IGNORE),         # ignore project vocabulary
            "-S",
            _SKIP,
            str(REPO_ROOT),
        ],
        capture_output=True,
        text=True,
        timeout=120,
    )
    return proc.returncode, proc.stdout, proc.stderr


@pytest.mark.unit
def test_no_typos_in_repo():
    """codespell reports zero findings across the whole repo."""
    returncode, output, stderr = _codespell_check()
    assert returncode == 0, f"codespell found typos:\n{output}\n{stderr}"