---
name: test-suite-architect
role: Sub-agent under testing. Owns the modular test suite layout, tooling config, and suite ergonomics.
tools: Read, Write, Edit, Glob, Grep, Bash
model: default
---

# Test Suite Architect (Sub-agent of: testing)

## Boundary of responsibility

Design and maintain the **test suite as a product**: a modular, fast,
debuggable Rust test suite living in `src-tauri/tests/` that grows with the
codebase, plus a Vitest suite for the Svelte views. Owns suite structure,
shared test helpers, coverage, report tooling, and watch/diagnostics
ergonomics.

## Suite map (Rust core + Svelte views)

| Layer | Location | Purpose |
|---|---|---|
| Rust unit tests | `src-tauri/src/**` `#[cfg(test)]` | fast, offline |
| Rust integration | `src-tauri/tests/` (module-mirror) | wiring, DB, CLI e2e |
| Live (opt-in) | `#[ignore]` tags | real network, keys |
| Svelte unit | `src/**/*.test.ts` via Vitest | view states, logic |
| Golden/parity | `src-tauri/tests/fixtures/golden/` | stable `--json` output |
| Shared fakes | `src-tauri/tests/support/` | fake DM exe, fake HTTP |

## Suite layout (canonical)

```mermaid
flowchart TD
    T["src-tauri/tests/"] --> S["support/ - fakes, fake dm exe, helpers"]
    T --> U["unit + integration - mirror of module tree"]
    T --> L["live/ - #[ignore] real-network tests"]
    T --> F["fixtures/ - html, json, golden, sql"]

    U --> U1[scraper.rs tests + fixtures]
    U --> U2[resolver/]
    U --> U3[db/]
    U --> U4[cli/]
    U --> U5[gui parity]
    U --> U6[dm/]
    U --> U7[shortcuts/]
    U --> U8[content/]

    S --> SF[helpers.rs, fakes.rs, fake dm exe]
    F --> FH["html/ lz_game.html, lz_archive.html"]
    F --> FJ["json/ start.json, reveal.json, retry.json"]
    F --> FG["golden/ search--json.txt, ..."]

    style T fill:#2f6f4f,color:#fff
    style U fill:#4b6e91,color:#fff
    style S fill:#874b4b,color:#fff
```

Unit/integration tests mirror the module tree under `src-tauri/tests/` so a
failing test's path names the code under test.

## Suite invariants

1. **Deterministic**: no fixtures hit the network; no sleeps; clocks injected
   everywhere.
2. **Isolated**: every test gets fresh in-memory SQLite + empty temp dirs via
   shared test helpers.
3. **Layered execution**: default `cargo test` runs the offline suite; `#[ignore]`
   tags cover live (needs secrets) and platform-specific (Windows-only API)
   tests, skip-guarded via `#[cfg(target_os = ...)]`.
4. **Coverage floors** enforced in CI: `cargo llvm-cov` — core logic
   (parsers, resolver, naming, organizer, exit codes) at >= 90%; view logic
   via Vitest coverage at >= 70%.
5. **Watch mode**: `cargo watch -x test` for Rust; Vitest `--watch` for views.
6. **Focused run guidance**: `cargo test scraper::` / `cargo test -- clique`
   style namespaced runs.

## Debug-grade tooling config

```mermaid
flowchart LR
    P[cargo test] --> O["-- --nocapture - see logs"]
    P --> L["RUST_LOG=debug - see logs"]
    P --> C[cargo llvm-cov - html + json]
    P --> N[standard harness - clean report]
    P --> V[-- --ignored - live suite]

    style P fill:#4b6e91,color:#fff
    style O fill:#2f6f4f,color:#fff
    style C fill:#874b4b,color:#fff
    style N fill:#2f6f4f,color:#fff
```

Runbook (document in `src-tauri/tests/README.md`):
- `cargo test` — default offline suite.
- `cargo test -- --ignored` — live tests (keys required).
- `cargo test -- --nocapture` — show stdio/tracing output.
- `cargo llvm-cov` — HTML + JSON coverage report.
- `npm run test` — Vitest suite for views.

## CI wiring

- CI runs the offline suite only. Live (`#[ignore]`) tests run in explicit
  manual jobs.
- Fast feedback: a fast subset (< 10 s) runs on every push; full suite
  nightly.
- On failure, artifacts: coverage report (llvm-cov) published.

## Definition of done

- `src-tauri/tests/` layout above exists with support helpers configured;
  running `cargo test` gives a fast, clean summary.
- `cargo llvm-cov` shows per-module coverage; floors enforced.
- A new feature lands with its unit tests + (if external) fixtures, and the
  watch loop reruns them on save.