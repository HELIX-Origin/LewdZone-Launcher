---
name: gui-conventions
rule_number: "13"
scope: tauri desktop app, cli/gui parity, view conventions
enforcement: app-shell + view-designer + cli agent; cli/gui parity checks
---

# Rule 13: GUI Conventions

The GUI is a **Tauri 2 desktop app** — the primary product. It is one process
with a Svelte webview and a Rust core. The same Rust binary also exposes a
**native CLI** (Rule 03): two entry points, one core. The webview never
touches the site, download dispatch, SQLite, or content providers directly —
every workflow goes through `invoke()` → Rust commands, and every command has
a CLI twin in `cli.rs`.

## Entry-point model (one process, two faces)

- **App mode** (`main.rs` → `lib.rs::run()`): windowed app; webview renders
  Svelte; Svelte calls `invoke('command_name', args)`; Rust executes the core
  function and returns serialized data.
- **CLI mode** (`main.rs`, no window): args parsed → the **same core
  functions** via `cli.rs` handlers → results printed, process exits.
- **Long-running work** (sync / download / shortcuts): the GUI runs it inside
  Tauri async commands and streams progress to the webview via channels; the
  CLI prints progress lines. Same core functions drive both.
- **No subprocess framing.** There is no Python, no sidecar binary, no
  JSON/JSONL hand-off between two programs. The Rust core IS the engine.

## CLI / GUI parity

1. Every GUI action maps 1:1 to a CLI subcommand (`--help` lists them all).
2. A GUI-only workflow with no CLI twin (or vice versa) is a design defect —
   the shared core must serve both.
3. Version is defined once (Cargo.toml) and synced to `tauri.conf.json` and
   root `package.json` (Rule 08); CLI `--version` == app version.

## View conventions

- **Pages:** Store (search + browse + download, Steam-style grid), Library
  (installed games: icon, cover, description, launch/shortcuts/uninstall),
  Downloads (queue), Settings.
- **Content comes live from the site** via the scrape pipeline — never
  hardcoded fixtures in the UI.
- **States:** each view has exactly loading / ready / error; no domain state
  survives navigation; virtualized lists; keyboard-first navigation.
- **Art:** cover-art-first tiles, equal aspect ratio; cached via the artwork
  pipeline (offline tolerant). See
  [view-designer](../agents/gui/view-designer/view-designer.md).

## Freeze checklist (per view feature)

- [ ] GUI path hits a Rust command in `lib.rs` (never an ad-hoc webview path)
- [ ] the same workflow exists as a `cli.rs` subcommand over the same core fn
- [ ] UI thread never blocks; long ops stream progress and are cancelable
- [ ] command output shape defined once, used by both GUI and CLI
- [ ] app version == CLI version (Rule 08)