"""CLI entry: argparse command engine (single source of truth).

Logic lives in controllers; the parser only maps args to controller calls and
prints machine JSON when ``--json`` is set. See
``.agents/agents/cli/command-designer/`` for the command spec.
"""

from __future__ import annotations

import argparse
import sys
from collections.abc import Sequence

from lewdzone_launcher import __version__
from lewdzone_launcher.controllers.game import GameController
from lewdzone_launcher.controllers.download import DownloadController
from lewdzone_launcher.controllers.sync import SyncController


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="lewdzone-launcher",
        description="LewdZone Launcher CLI — engine for the desktop app.",
    )
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")
    parser.add_argument("--json", action="store_true", help="emit a single JSON document")
    parser.add_argument("--jsonl", action="store_true", help="emit JSONL events for long operations")

    sub = parser.add_subparsers(dest="command", required=True)

    g = sub.add_parser("game", help="details for a game")
    g.add_argument("--game", required=True, help="game slug")

    d = sub.add_parser("download", help="enqueue a download via the active download manager")
    d.add_argument("--game", required=True)
    d.add_argument("--version", required=True)
    d.add_argument("--platform", required=True, choices=["PC", "Android", "Linux", "Mac"])
    d.add_argument("--tab", choices=["official", "community"], default="official")
    d.add_argument("--manager", choices=["fdm", "idm", "torrent"], default=None)

    sub.add_parser("sync", help="synchronize the catalog into the local database")

    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    if args.command == "game":
        return GameController().info(slug=args.game, as_json=args.json)
    if args.command == "download":
        return DownloadController().enqueue(
            game_slug=args.game,
            version=args.version,
            platform=args.platform,
            tab=args.tab,
            manager=args.manager,
            as_json=args.json,
        )
    if args.command == "sync":
        return SyncController().run(as_json=args.json)

    parser.print_help(sys.stderr)
    return 2