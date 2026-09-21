"""Live verification of the archive pagination scheme (promoted from
scratch/probe_pagination.py -- see tests/unit/scanners/test_scratch_promotion.py).

Resolved empirically: lewdzone.com/games uses WordPress pretty permalinks
``/games/page/N/`` with a trailing slash. Query forms ``?page=N`` and
``?paged=N`` are ignored by WordPress (normalized back to the base page).
Filters combine on the page path, e.g.
``https://lewdzone.com/games/page/2/?platform=PC``. The archive shows 20
game links per page; the pagination widget last-page anchor is
``/games/page/1146/`` (snapshot at the time this test was written).

Opt-in: run with ``python -m pytest -m live``.
"""

from __future__ import annotations

import re

import pytest
import urllib.request

from tests.conftest import REPO_ROOT  # noqa: F401  (documents repo layout for readers)

GAMES = "https://lewdzone.com/games/"
UA = {"User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)"}


def _fetch(url: str) -> bytes:
    req = urllib.request.Request(url, headers=UA)
    with urllib.request.urlopen(req, timeout=20) as resp:
        return resp.read()


def _canonical(body: bytes) -> str | None:
    m = re.search(r'rel="canonical"[^>]*href="([^"]+)"', body.decode("utf-8", "replace"))
    return m.group(1) if m else None


def _game_slugs(body: bytes) -> set[str]:
    return set(re.findall(r"/game/([a-z0-9-]+)/", body.decode("utf-8", "replace")))


@pytest.mark.live
def test_archive_page_contains_game_links():
    body = _fetch(GAMES)
    assert len(_game_slugs(body)) >= 5, "archive page should list game tiles"


@pytest.mark.live
def test_query_page_form_is_ignored():
    body = _fetch(GAMES + "?page=2")
    assert _canonical(body) == GAMES, "?page=2 must normalize back to the base page"


@pytest.mark.live
def test_pretty_page_path_is_canonical():
    body = _fetch(GAMES + "page/2/")
    assert _canonical(body) == GAMES + "page/2/"
    assert len(_game_slugs(body)) >= 5


@pytest.mark.live
def test_pagination_links_present():
    body = _fetch(GAMES + "page/2/")
    text = body.decode("utf-8", "replace")
    assert "/games/page/3/" in text
    assert "/games/page/1146/" in text, "last-page anchor should exist (1146 snapshot)"


@pytest.mark.live
def test_filters_combine_on_page_path():
    body = _fetch(GAMES + "page/2/?platform=PC")
    assert _canonical(body).startswith(GAMES + "page/2/")
    assert len(_game_slugs(body)) >= 1