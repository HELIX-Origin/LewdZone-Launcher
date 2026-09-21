---
name: code-style-rust
rule_number: "01"
scope: all rust source
enforcement: cargo fmt --check, cargo clippy -D warnings, cargo check
---

# Rule 01: Rust Code Style

> Historical note: the `-python` filename is kept for path stability; this
> rule governs **Rust** code style only.

Every `.rs` byte in lewdzone-launcher must pass `cargo fmt` (format) and
`cargo clippy` with `-D warnings` (lint). Style is enforced by tooling, not by
taste.

## Tooling baseline

| Tool | Config anchor | Enforced |
| --- | --- | --- |
| `cargo fmt` | rustfmt defaults | formatting |
| `cargo clippy -- -D warnings` | curated allowlist on `src-tauri/src/lib.rs` | lint |
| `cargo check` / `cargo test` | `Cargo.toml` | types + tests |

## Format rules

1. rustfmt defaults: 4-space indent, module order (declarations → imports →
   items), rustfmt-compatible line breaking.
2. Imports: `use` grouped and sorted; no wildcard `*` imports inside `src/`.
3. No unused imports/items (`cargo clippy` flags `unused_imports`).
4. Prefer `std::path::PathBuf` over `String` for filesystem paths.

## Naming (see also Rule 02)

- Modules/files: `snake_case`.
- Types/structs/enum variants: `PascalCase`. Functions/methods/variables:
  `snake_case`.
- Constants: `UPPER_SNAKE`.
- Non-`pub` items are crate-private; keep the public API surface minimal.

## Types

1. Every `pub` function has explicit parameter and return types.
2. Domain models are plain structs/enums with explicit derives (see Rule
   02/03 canonical models).
3. No `unwrap()`/`expect()` outside tests and top-level entry points — use `?`
   with typed errors ([Rule 12](./rule-12-error-handling.md)).
4. Newtype wrappers for domain primitives where unit confusion is possible
   (e.g. `struct PostId(u64)`, `struct GoToken(String)`).
5. `Option<T>` for possibly-absent values, never sentinel strings.
6. `enum` variants over flag booleans for modes (e.g. `SortMode`, `Platform`,
   `TabId`).

## Anti-patterns (fail clippy/lint)

```rust
// BAD: global mutable state, panic in library code, stringly-typed tokens
static STATE: std::sync::Mutex<HashMap<String, String>> = /* … */;
let _parts: Vec<&str> = url.split("#t=").collect(); // stringly-typed token handling
pub fn handle(thing: &str) -> String { thing.to_string() } // stringly-typed
```

```rust
// GOOD
#[derive(Debug, Clone)]
pub struct DownloadEntry {
    pub platform: Platform,
    pub tab: DownloadTab,
    pub label: String,
    pub token: GoToken, // typed domain value object
}
```

## Verify

From `src-tauri/`:

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo check`
- `cargo test`