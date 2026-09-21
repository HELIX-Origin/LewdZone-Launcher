"""Module entry point: ``python -m lewdzone_launcher``."""

from __future__ import annotations

from lewdzone_launcher.frontends.cli.parser import main

if __name__ == "__main__":
    raise SystemExit(main())