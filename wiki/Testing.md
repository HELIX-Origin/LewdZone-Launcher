# Testing & QA

> Links between wiki pages are relative and omit the `.md` extension.

## Layers

| Layer | What it covers | How it runs |
| --- | --- | --- |
| **Unit** | pure logic: parsers, formats, domain, organizers | `cargo test` in `src-tauri/` — offline, fast |
| **Integration** | controllers, db with scratch DB, resolver/adapters | `cargo test`; no live network |
| **Live** | real `lewdzone.com`, SteamGridDB, real FDM | opt-in `cargo test -- --ignored` (tagged `#[ignore]`) |
| **GUI tests** | Svelte views against the Rust core | Vitest (`npm run test`) in `src/` |
| **Windows-specific** | spawn flags, `.lnk`, path rules | `cargo test` on a Windows CI runner |

## Running the suite

```sh
cargo test                 # from src-tauri/: default offline-only
cargo test -- --ignored    # opt-in live networking
npm run test               # from repo root: Vitest frontend suite
```

Coverage floors: **85% overall**, core modules ~90%, GUI-side ~70% (via
`cargo llvm-cov` when available). Gate in CI: `cargo fmt --check`,
`cargo clippy -D warnings`, `cargo test`, `npm run check`, `npm run test`. See
[Rule 11](../.agents/rules/rule-11-testing).

## Test layout

```
src-tauri/
  src/                  # inline #[cfg(test)] unit tests per module
  tests/
    integration/        # cross-module tests
    live/               # requires #[ignore] (opt-in)
    fixtures/           # html/json/golden/sql fixtures
    support/            # test helper modules (fakes, never shipped)
    perf/               # micro-benchmarks (criterion)
src/
  lib/                  # Vitest suites per view/component (*.test.ts)
```

## Fakes (test helper modules)

The suite never touches real anything by default. Key fakes:

| Fake | Stands in for |
| --- | --- |
| fake HTTP transport (`support/http.rs`) | real site/api, canned by URL |
| DM exe shims (per manager) | FDM / IDM / torrent, record argv to file |
| fake SteamGrid (`support/steamgrid.rs`) | SteamGridDB artwork API |
| command-test harness | core commands called in-process, assert on results |
| shortcut fakes | `.lnk`, `.desktop`, macOS alias |

Fakes **fail loud** on unexpected input so bugs aren't masked. Documented in
[the agent](../.agents/agents/testing/mock-engineer/mock-engineer).

## Protocol & parity tests

- A **parity test** exercises `lewdzone <cmd> --json` and asserts the
  CLI's machine output stays 1:1 with the GUI-facing core commands (GUI/CLI
  drift = bug).
- Long-running commands are tested by feeding canned data through the
  in-process command test harness and asserting state transitions — no
  subprocess involved (Rule 13).

## Probe/script promotion (reuse, don't remake)

The Rust test suite is the **sole home for every script in this repo** —
scanning, probing, verification, and debugging logic all live as test modules,
never as scripts in `scratch/`:

- Offline checks → unit/integration tests in `src-tauri/`.
- Real-network probes → `live/` tests tagged `#[ignore]` (opt-in, keeps the
  offline gate hermetic).
- Benchmarks → `src-tauri/tests/perf/` criterion benches.
- Scratch scanners are promoted into `src-tauri/` tests; the gate fails if
  stray logic appears in `scratch/` — the fix is to promote it into tests.
- Empirical findings (e.g. archive pagination = `/games/page/N/`, 20
  games/page) are recorded in the ROADMAP **and** pinned as live tests so they
  never regress silently.

## Performance budgets

| Operation | Budget |
| --- | --- |
| cold start | <2s |
| `list` | <300ms |
| `search` | <200ms |
| page parse | <400ms |
| site fan-out | ~1 req/s (throttled) |

Tracked in `src-tauri/tests/perf/` with micro-benchmarks; N+1 queries are rejected at
review. See [Rule 11](https://github.com/helix-origin/lewdzone-launcher/tree/main/.agents/rules/rule-11-testing.md) and
[the perf-auditor](https://github.com/helix-origin/lewdzone-launcher/tree/main/.agents/agents/review/perf-auditor/perf-auditor.md).