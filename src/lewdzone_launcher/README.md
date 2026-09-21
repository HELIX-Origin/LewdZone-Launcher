# LewdZone Launcher

## Package

```
src/lewdzone_launcher/
  __init__.py            # __version__
  __main__.py            # python -m lewdzone_launcher entry
  _config.py             # per-OS config/db paths
  frontends/cli/         # argparse command engine
    __init__.py
    parser.py
  controllers/           # game, download, sync, shortcut, artwork
    __init__.py
  domain/                # stdlib-only core models
    __init__.py
  services/
    scraping/
    resolver/
    db/
    dm/
      adapters/{fdm,idm,torrent}/
    shortcuts/
    artwork/
```

## Conventions

- `src` layout, installable via `pip install -e .`
- import-linter layer contract (Rule 03)
- Ruff + Pyright strict (Rule 01)
- Naming: snake_case modules, `XController` controllers, enums in `domain`
  (Rule 02)
- Cross-platform: `pathlib` everywhere, per-OS config dirs from `_config.py`,
  per-OS spawn flags in services (Rule 07)

See [AGENTS.md](../../AGENTS.md) and `.agents/rules/` for the full contract.