# ADR-0002: Download dispatch (supersedes download-manager adapter layer)

- **Status:** superseded (2026-09-23)
- **Date:** 2026-09-21
- **Owner:** download family (formerly dm family)
- **Applies to:** Rule 03, Rule 07, Rule 12

## Context

The original design handed every resolved real URL to an installed download
manager (FDM, IDM, or a torrent client) through a pluggable adapter layer.
Detection proved unreliable across versions and platforms, and every supported
host already has a viable in-app or OS-native path. The DM layer was removed in
favor of a simpler dispatch model.

## Decision

Downloads are dispatched by host class:

- **Direct-file hosts** (`fileknot` today) stream in-app with byte progress to
  the staging folder (`<downloads>/Games/<Title>/`), then auto-extract `.zip`
  archives into `<lzapps>/<slug>/` and write an `app.json` manifest.
- **All other hosts** hand the resolved URL to the OS default handler
  (`core::native::open_url`). This opens the installed cloud app or browser
  with zero configuration.

No registry of external download managers is kept. There is no `dm` setting.
Exit code 4 (download manager missing) is reserved/unused.

### Seams

- `core::download::dispatch_with(job, stream, progress)` — routes by host class.
- `core::extract::install_from_archive(...)` — zip extraction + `app.json`.
- `core::native::open_url(...)` — OS default handler, no shell.

## Consequences

- **Benefits:** zero download-manager setup; no per-manager detection or argv
  drift; cross-platform by default; direct downloads get live progress.
- **Costs/risks:** page-gated hosts (workupload, mixdrop) open in the browser
  and require the user to click; non-zip archives stay in `downloads/` and must
  be handled manually.
- **Migration:** forward-only; the old adapter files were deleted.

## Verification

- [x] Contract documented in `dm.md`, `rule-07-download-manager-integration.md`
      (renamed Rule 07: Download Dispatch), and wiki `Download-Managers.md`.
- [x] Stream redirect tests cover same-owner validation.
- [x] Extraction + manifest tests cover zip install and replacement updates.
- [x] Launch tests cover `launch_exe` override and candidate fallback.
