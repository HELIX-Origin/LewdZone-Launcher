"""Download controller: enqueue jobs to the active download manager."""

from __future__ import annotations

from lewdzone_launcher.domain.exceptions import LewdzoneError


class DownloadController:
    def enqueue(
        self,
        game_slug: str,
        version: str,
        platform: str,
        tab: str = "official",
        manager: str | None = None,
        as_json: bool = False,
    ) -> int:
        raise LewdzoneError("not implemented yet")