"""Sync controller: catalog synchronization pipeline."""

from __future__ import annotations

from lewdzone_launcher.domain.exceptions import LewdzoneError


class SyncController:
    def run(self, as_json: bool = False) -> int:
        raise LewdzoneError("not implemented yet")