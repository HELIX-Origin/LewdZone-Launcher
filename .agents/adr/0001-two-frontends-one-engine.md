# ADR-0001: Two frontends, one engine

- **Status:** accepted
- **Date:** 2026-09-21
- **Owner:** architect / cli
- **Applies to:** Rule 03, Rule 12, Rule 13

## Context

The product must ship a desktop GUI and a scriptable engine. Both entry points
must share the same Rust core logic and a stable contract so that neither the
GUI webview nor the CLI re-implements behavior.

## Decision

- The **CLI** (`lewdzone <cmd> [options]`) is the scriptable headless entry point
  into the same Rust core.
- The **app** (Tauri 2, Rust core + Svelte webview) calls the same core
  functions in-process via `#[tauri::command]` handlers. The webview never
  touches the site, DB, or network directly.
- **Query commands** (short, bounded): one invocation → one JSON document on
  stdout (`--json`). Process is reaped immediately.
- **Long-running commands** (sync / download / shortcuts): `--json` emits a
  single JSON result on completion; human mode prints progress on stderr.
- `stdout` is the protocol channel; `stderr` is diagnostics only and never
  parsed as data.
- Exit codes are authoritative per Rule 12 (0 ok, 1 runtime, 2 usage, 3
  network, 4 configuration/dependency, 5 interrupted).
- Subprocess rules: hidden console on Windows (`CREATE_NO_WINDOW`), detached
  session on POSIX, explicit argument arrays, never a shell string.

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

- [x] Contract documented in `cli.md`, `gui.md`, Rule 03, Rule 13
- [x] `lewdzone --help` / `--version` smoke
- [x] Wiki `Architecture` + `CLI-Reference` updated