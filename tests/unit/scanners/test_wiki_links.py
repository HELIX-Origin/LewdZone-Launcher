"""Scanner: wiki internal links follow the repo convention.

Wiki pages must link to other wiki pages with RELATIVE paths and NO ``.md``
extension (e.g. ``[Home](Home)``, not ``[Home](Home.md)`` or absolute URLs),
so the GitHub wiki and the local ``wiki/`` tree stay interchangeable.
"""

from __future__ import annotations

import re
from pathlib import Path

import pytest

from tests.conftest import REPO_ROOT

WIKI_DIR = REPO_ROOT / "wiki"

_MDLINK = re.compile(r"\]\(([^)]+)\)")


@pytest.mark.unit
def _wiki_files():
    return sorted(WIKI_DIR.glob("*.md"))


@pytest.mark.unit
def test_wiki_md_files_exist():
    assert len(_wiki_files()) >= 15
    assert (WIKI_DIR / "Home.md").exists()


@pytest.mark.unit
@pytest.mark.parametrize(
    "path",
    _wiki_files(),
    ids=lambda p: p.name,
)
def test_wiki_links_are_relative_without_extension(path: Path):
    """Every ``](target)`` in wiki pages is relative and lacks ``.md``."""
    text = path.read_text(encoding="utf-8", errors="replace")
    for m in _MDLINK.finditer(text):
        target = m.group(1)
        if "://" in target:  # external http(s) links are fine
            continue
        if target.startswith("../"):  # repo-tree links keep their real .md path
            continue
        if target.endswith("/"):  # directory-style target (footer/home) tolerated
            continue
        assert not target.endswith(".md"), f"{path.name}: link has .md ext: {m.group(0)}"
        assert not target.startswith("wiki/"), f"{path.name}: link is not relative: {m.group(0)}"
        assert not target.startswith("./"), f"{path.name}: link has ./ prefix: {m.group(0)}"