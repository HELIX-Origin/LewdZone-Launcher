"""Scanner: reject stale references left behind by renames.

Guards against the family of regressions hit during refactors: the project was
renamed ``lewdzone-cli``→``lewdzone-launcher`` and the FDM-only family was
replaced by a pluggable ``dm`` (download-manager) layer. Any doc that still
mentions the old names, or an old family/package token, is a defect.
"""

from __future__ import annotations

import pytest

from tests.conftest import REPO_ROOT, iter_repo_files

_STALE_REF = {
    "lewdzone_cli": "package was renamed to lewdzone_launcher",
    "lewdzone-cli": "binary was renamed to lewdzone-launcher",
    "src/lewdzone_launcher/frontends/gui": "gui is a Tauri app in desktop/, not a frontend package",
    "agents/cli/fdmcmd": "fdmcmd command was removed",
}

# Files that must never contain a bare fdm/DM-family mismatch token.
_FDM_STALE = {
    ("lewdzone_launcher/services/fdm", "services layer is services/dm"),
    ("agents/fdm", "the fdm agent family was replaced by agents/dm"),
}


@pytest.mark.unit
@pytest.mark.parametrize(
    "path",
    iter_repo_files(),
    ids=lambda p: str(p.relative_to(REPO_ROOT)),
)
def test_no_stale_rename_refs(path):
    """Repo files never mention old project/binary/family names."""
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return
    hits = {needle for needle in _STALE_REF if needle in text}
    assert not hits, f"{path}: stale refs {list(hits)}"


@pytest.mark.unit
def test_dm_layer_dir_stale_free():
    """No agent doc still points at a services/fdm or agents/fdm path token."""
    files = [p for p in iter_repo_files() if p.suffix == ".md"]
    for path in files:
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for needle, hint in _FDM_STALE:
            assert needle not in text, f"{path}: {needle} ({hint})"


@pytest.mark.unit
def test_pypi_package_name_not_present_where_binary_expected():
    """Package imports reference the underscore name only."""
    for path in iter_repo_files((".py",)):
        text = path.read_text(encoding="utf-8", errors="replace")
        assert "from lewdzone-cli" not in text
        assert "import lewdzone-cli" not in text