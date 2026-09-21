"""Game controller: catalog queries and game details."""

from __future__ import annotations

from lewdzone_launcher.domain.exceptions import LewdzoneError


class GameController:
    def info(self, slug: str, as_json: bool = False) -> int:
        raise LewdzoneError("not implemented yet")