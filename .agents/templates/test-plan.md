---
name: test-plan
description: Template for a new feature test plan
---

# Test Plan: {{ Feature }}

## Scope

{{ What is being tested. }}

## Unit tests

- [ ] {{ Rust unit test: module, scenario, expected result }}
- [ ] {{ Rust unit test }}

## Integration tests

- [ ] {{ CLI command output verification }}
- [ ] {{ Tauri command via frontend test }}

## Manual / live tests

- [ ] {{ Live site scenario }}
- [ ] {{ UI interaction scenario }}

## Regression checks

- [ ] `cargo fmt --check`
- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo test`
- [ ] `npm run check`
- [ ] `npm run test`

## Known limitations

{{ Any flaky live tests or skipped coverage. }}
