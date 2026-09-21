"""Scanner: no Python scripts live in scratch/ — the pytest suite is the sole
home for every script (scanning, probing, debugging, verification, utilities).

Convention (see rule-11-testing and wiki/Testing): ALL scripts used in this
repo are authored as pytest modules inside ``tests/`` (``tests/unit`` for
offline logic, ``tests/live`` for opt-in network probes, ``tests/perf`` for
benchmarks). ``scratch/`` holds no scripts at all — anything that needs a
script is promoted into the suite instead of being re-created each session.

This guard fails on ANY ``.py`` file under ``scratch/``.
"""

from __future__ import annotations

import pytest

from tests.conftest import REPO_ROOT


@pytest.mark.unit
def test_scratch_has_no_python_scripts():
    scratch = REPO_ROOT / "scratch"
    offenders = sorted(p.name for p in scratch.glob("*.py")) if scratch.is_dir() else []
    assert not offenders, (
        "scratch/ must contain no .py files; the pytest suite is the sole home "
        "of scripts. Promote the logic into tests/ (unit/integration/live/perf) "
        "and delete the scratch file(s): "
        + ", ".join(offenders)
    )


@pytest.mark.unit
def test_promoted_live_tests_exist():
    """Sanity: the promoted archive-pagination probe lives in tests/live."""
    live = REPO_ROOT / "tests" / "live"
    assert (live / "test_archive_pagination.py").is_file()