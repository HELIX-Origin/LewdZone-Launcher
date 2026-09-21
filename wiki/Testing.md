# Testing & QA

> Links between wiki pages are relative and omit the `.md` extension.

## Layers

| Layer | What it covers | How it runs |
| --- | --- | --- |
| **Unit** | pure logic: parsers, formats, domain, organizers | default `pytest` — offline, fast |
| **Integration** | controllers, db with scratch DB, resolver/adapters | default; no live network |
| **Live** | real `lewdzone.com`, SteamGridDB, real FDM | opt-in `pytest -m live` |
| **GUI tests** | Tauri-side protocols (parity) | `pytest -m gui` |
| **Windows-specific** | spawn flags, `.lnk`, path rules | `pytest -m windows` |

## Running the suite

```sh
pytest -q                # default: offline-only
pytest -m live           # opt-in live networking
pytest --cov --cov-fail-under=85
pytest --lf --ff         # rerun last failures first
```

Coverage floors: **85% overall**, core modules ~90%, GUI-side ~70%. Gate in CI:
`ruff check`, `pyright`, `pytest -q`, bandit + pip-audit. See
[Rule 11](../.agents/rules/rule-11-testing).

## Test layout

```
tests/
  conftest.py            # shared fixtures + markers
  unit/                  # mirror of src/lewdzone_launcher/
  integration/
  live/                  # requires -m live
  fixtures/              # html/json/golden/sql fixtures
  support/               # fakes (never shipped)
  perf/                  # micro-benchmarks
```

## Fakes (tests/support)

The suite never touches real anything by default. Key fakes:

| Fake | Stands in for |
| --- | --- |
| `FakeHttpTransporter` | real site/api, canned by URL |
| DM exe shims (per manager) | FDM / IDM / torrent, record argv to file |
| `FakeSteamGrid` | SteamGridDB artwork API |
| `FakeSpawn / sidecar runner` | the real app→CLI subprocess, emits canned JSONL |
| shortcut fakes | `.lnk` (win32com), `.desktop`, macOS alias |

Fakes **fail loud** on unexpected input so bugs aren't masked. Documented in
[the agent](../.agents/agents/testing/mock-engineer/mock-engineer).

## Protocol & parity tests

- A **parity test** exercises `lewdzone-launcher <cmd> --json` and asserts the
  app's bridge contract stays 1:1 with the CLI (app/CLI drift = bug).
- Long-running commands are tested by feeding canned JSONL streams through the
  fake sidecar runner and asserting UI-state transitions.

## Performance budgets

| Operation | Budget |
| --- | --- |
| cold start | <2s |
| `list` | <300ms |
| `search` | <200ms |
| page parse | <400ms |
| site fan-out | ~1 req/s (throttled) |

Tracked in `tests/perf/` with micro-benchmarks; N+1 queries are rejected at
review. See [Rule 11](../.agents/rules/rule-11-testing) and
[the perf-auditor](../.agents/agents/review/perf-auditor/perf-auditor).