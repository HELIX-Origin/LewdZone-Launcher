---
name: module-architecture
rule_number: "03"
scope: repo layout, module boundaries, contracts
enforcement: struct review by architect + review gate + ADR requirement
---

# Rule 03: Module Architecture

lewdzone-launcher is a **single Tauri 2 application** (Rust core + Svelte
webview) that also ships a **native Rust CLI**. The CLI and the GUI are two
entry points into the same core: the same functions back both
(`src-tauri/src/cli.rs` subcommands and `src-tauri/src/lib.rs` commands).
There is **no Python anywhere**. There is no embedded/scripted CLI engine and
no `desktop/` folder — the app root IS the repo root (Tauri standard layout).

## Real repo layout

```
lewdzone-launcher/
  .agents/                 # agent ecosystem (rules, agents, skills, templates, adr/)
  src/                     # Svelte + SvelteKit webview (frontendDist = ../build)
    routes/
    app.html
    +layout.ts
    +page.svelte
  src-tauri/               # Rust core + native CLI
    src/
      lib.rs               # tauri commands + run()
      main.rs              # entry: CLI dispatch OR app run
      cli.rs               # native subcommand handlers (shared with GUI)
    Cargo.toml
    tauri.conf.json        # app window, bundle, cli schema (if used)
    capabilities/
    icons/                 # tauri icon set (icon.ico/.icns/pngs, from assets/icon.png)
  assets/                  # app icon source + misc assets
  static/                  # webview static files
  wiki/                    # THE documentation home (no docs/ folder)
  package.json             # root: Svelte build + @tauri-apps/cli
  .gitignore
```

## Two entry points, one core

| Entry point | Runs from | Context |
| --- | --- | --- |
| Desktop app | `src-tauri/` binary, windowed | Webview → Svelte → Tauri `invoke()` → `lib.rs` commands |
| CLI | same binary invoked non-windowed | Args parsed → `cli.rs` handlers call the **same core functions** as the GUI |

- `main.rs`: if CLI args present → dispatch to `cli.rs` and exit; otherwise →
  `lib.rs::run()` starts the windowed app.
- **GUI/CLI parity (Rule 13):** a workflow implemented only in the GUI and not
  reachable from the CLI — or only in the CLI and not in the GUI — is a
  violation. Both must hit the same core functions.
- The webview never talks to the network, SQLite, or the site directly.
  Everything goes through Tauri `invoke()` → Rust commands.

## Contracts & ADRs

1. **Every cross-layer interface is a contract**: documented signature, typed,
   tested at its owner.
2. **Contract change requires an ADR** (see
   [architect](../agents/architect/architect.md) +
   [ADR template](../templates/adr.md)) before implementation. ADRs live in
   `.agents/adr/`.
3. **Command parity contract:** every Rust `#[tauri::command]` has a matching
   CLI subcommand handler in `cli.rs` calling the same core; a GUI-only code
   path that bypasses the shared core is a Rule 03 + Rule 13 violation.

## Boundaries

| Layer | Contains | May import |
| --- | --- | --- |
| `src-tauri/src/` | `lib.rs` (commands), `cli.rs` (CLI), core modules (db, scrape, resolve, content, download, queue, folder, native, artwork…) | each other + crate deps |
| `src/` (Svelte) | webview views | Tauri `invoke()` only — never Rust internals directly |
| external | lewdzone.com, sqlite, SteamGridDB/VNDB/IGDB | — |

- Keep the Rust core deps small and well-justified; prefer `tokio` (Tauri
  default) + platform crates over heavy frameworks.
- No `unsafe` without a comment + review gate sign-off.
- Shortcuts use native mechanisms per OS (`win32com` .lnk, `.desktop`, `.app`
  aliases — see [shortcuts](../agents/shortcuts/shortcuts.md)).

## Verify

- `cargo build` / `cargo check` / `cargo test` green in `src-tauri/`
- `npm run tauri dev` / `npm run tauri build` green
- `npm run build:installer` produces the unified installer artifact
- `src-tauri/target/release/lewdzone` answers `--help` AND launches the app
  when run bare — CLI path and GUI path both smoke-test.