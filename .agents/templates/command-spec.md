---
name: command-spec
description: Template for a new CLI subcommand specification
---

# CLI Command Spec: `{{ command }}`

## Purpose

{{ One-line purpose. }}

## GUI parity

- Tauri command: `{{ invoke_command }}`
- Rust core function: `{{ core::module::run }}`

## Syntax

```
lewdzone {{ command }} [OPTIONS] [ARGS]
```

## Arguments

| Name | Required | Default | Description |
| --- | --- | --- | --- |
| {{ arg }} | {{ yes/no }} | {{ default }} | {{ description }} |

## Options

| Flag | Description |
| --- | --- |
| `--json` | Emit machine-readable JSON on stdout. |
| {{ custom flag }} | {{ description }} |

## Exit codes

- `0` — success
- `1` — runtime error
- `2` — usage error
- `3` — network/site error
- `5` — interrupted

## Examples

```bash
# Human output
lewdzone {{ command }} {{ example args }}

# JSON output
lewdzone {{ command }} {{ example args }} --json
```

## Implementation notes

- Add `{{ CommandVariant }}` to `cli.rs` `Command` enum.
- Dispatch to the shared core; do not implement logic in `cli.rs`.
- Add a parse test in `cli.rs` tests.

## Verification

- [ ] `lewdzone --help` lists the command cleanly.
- [ ] Parse test added.
- [ ] Core function has its own unit tests.
