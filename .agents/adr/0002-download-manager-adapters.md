# ADR-0002: Download-manager adapter layer

- **Status:** accepted
- **Date:** 2026-09-21
- **Owner:** dm family
- **Applies to:** Rule 03, Rule 07, Rule 12

## Context

The tool must not download files itself. Resolved real URLs (and magnets) go to
an installed download manager, and the supported set must be extensible beyond
FDM. Managers differ per OS: FDM/IDM are Windows-only; torrent clients exist on
all platforms. Spawn modes and silent flags differ.

## Decision

- A pluggable **DownloadManager adapter contract** under
  `src/lewdzone_launcher/services/dm/adapters/`:

  ```
  name: str
  platforms: tuple[str, ...]            # sys.platform subset
  handles_kind: Literal["http", "torrent"]
  detect() -> Path | None               # override, known paths, PATH, config dirs
  launch(url: str, target_dir: Path, filename: str) -> None
  ```

- Registry `MANAGERS: dict[str, DownloadManager]`; active manager chosen with
  `lewdzone dm <name>` (persisted to the `dm` setting).
- **Torrent** links route only to torrent-capable managers (`handles_kind`);
  HTTP adapters never receive magnets and torrent adapters never get HTTP URLs.
- **Resolution rule:** only the resolved real URL is handed over, never a
  `#fragment`. Trailing literal `\r` stripped from reveal responses.
- **Spawn:** `shell=False`, args array (no interpolation); Windows
  `CREATE_NO_WINDOW`, POSIX `start_new_session=True`; silent flags mandatory
  (FDM `-fs`, IDM `/n`, torrent positional).
- **Failure:** no manager installed → exit 4 listing alternatives; unhandled
  kind → typed error (Rule 12).

## Consequences

- **Benefits:** "more managers" = one new adapter file + registry entry +
  fake in `tests/support/`; per-manager argv shapes pinned by tests.
- **Costs/risks:** per-manager quirks (IDM `/p` dir flag); version drift of
  third-party CLIs; must re-verify detection paths.
- **Migration:** forward-only; new adapters are additive.

## Alternatives considered

1. Bake FDM calls directly into controllers — rejected: violates Rule 03
   layers, non-extensible, Windows-only.
2. One "spawn browser helper" — rejected: silent flags/detection are per-manager.

## Verification

- [x] Contract documented in `dm.md`, adapters, `rule-07`
- [ ] argv-shape unit tests per adapter (fakes in `tests/support/`, `-m live` opt-in)
- [x] Wiki `Download-Managers` updated