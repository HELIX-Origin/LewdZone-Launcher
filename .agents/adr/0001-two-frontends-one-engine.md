# ADR-0001: Two frontends, one engine

- **Status:** accepted
- **Date:** 2026-09-21
- **Owner:** architect / cli
- **Applies to:** Rule 03, Rule 12, Rule 13

## Context

The product must ship a desktop GUI and a scriptable engine. The GUI must not
import the Python package (the app embeds a PyInstaller sidecar of the same
CLI). We need a single source of truth for game logic and a stable cross-process
contract that both a human and the Tauri app can use.

## Decision

- The **CLI** (`lewdzone-launcher <cmd> [options]`) is the engine and the single
  source of truth. It exposes every app capability headlessly.
- The **app** (Tauri 2, Rust core + Svelte webview) never imports
  `lewdzone_launcher`. It spawns the CLI sidecar as a subprocess and parses
  machine output.
- **Query commands** (short, bounded): one invocation → one JSON document on
  stdout (`--json`). Process is reaped immediately.
- **Long-running commands** (sync / download / shortcuts): machine mode
  (`--jsonl`) streams newline-delimited JSON events
  `{"event","progress","message",...}` with a final `{"event":"result",...}`.
- `stdout` is the protocol channel; `stderr` is diagnostics only and never
  parsed as data.
- Exit codes are authoritative per Rule 12 (0 ok, 1 runtime, 2 usage, 3
  network, 4 dm missing, 5 interrupted).
- Spawn rules: hidden console on Windows (`CREATE_NO_WINDOW`), detached session
  on POSIX, `shell=False`, never a shell string.

## Consequences

- **Benefits:** one implementation, no logic in the webview, drift caught by
  parity tests, scriptable without the app.
- **Costs/risks:** subprocess overhead per command (~tens of ms); JSON parse
  strictness required; sidecar version must equal app version (Rule 08).
- **Migration:** every new GUI action must first exist as a CLI command.

## Alternatives considered

1. GUI imports the package directly — rejected: no packaging separation, no
   scripting boundary, makes sidecar embedding pointless.
2. HTTP localhost service between app and CLI — rejected: extra auth/port
   management, subprocess JSONL is simpler and matches one-command-per-action.

## Verification

- [x] Contract documented in `cli.md`, `gui.md`, `sidecar-driver.md`
- [x] `lewdzone-launcher --help` / `--version` smoke
- [ ] Parity tests (app bridge 1:1 with CLI) in Phase 3
- [x] Wiki `Architecture` + `CLI-Reference` updated