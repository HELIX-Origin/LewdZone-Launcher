# 🧪 Testing & Quality Assurance

> Links between wiki pages are relative and omit the `.md` extension.

LewdZone Launcher maintains a comprehensive, hermetic testing suite that enforces correctness across the Rust core, database migrations, scraper parsers, token resolver, download queue, archive extractors, and frontend views.

---

## 🧬 Test Layers

| Layer | Scope | Execution | Environment |
| --- | --- | --- | --- |
| **Rust Unit Tests** | Parsing, models, folder organization, 7-Zip path resolution, configuration logic | `cargo test` in `src-tauri/` | Offline, deterministic |
| **Rust Integration Tests** | SQLite migrations, catalog sync, download worker, cancellation, extraction | `cargo test` in `src-tauri/` | Isolated in-memory/temp SQLite DB |
| **Frontend Unit Tests** | Svelte views (`Store`, `Library`, `Downloads`, `Settings`, `Favorites`) | `npm run test` (Vitest) in repo root | Mocked Tauri IPC invoke handlers |
| **Type & Lint Checks** | Svelte compiler checks, Rust linter, formatting | `npm run check`, `cargo clippy -- -D warnings`, `cargo fmt --check` | Pre-merge verification gate |
| **Opt-In Live Tests** | Real network probes against `lewdzone.com` and content providers | `cargo test -- --ignored` | Tagged with `#[ignore]` |

---

## ▶️ Running the Suites

```bash
# Run all Rust offline unit and integration tests (from src-tauri/)
cargo test

# Run Rust formatting and clippy linter checks
cargo fmt --check
cargo clippy -- -D warnings

# Run all frontend Vitest tests (from repo root)
npm run test

# Run Svelte TypeScript and template checks
npm run check
```

---

## 🎭 Test Doubles & Fakes

To guarantee that the test suite runs hermetically and safely without internet connectivity:
- **HTTP Transport Fake:** Intercepts outgoing requests to return canned HTML and JSON fixtures for LewdZone catalog and game pages.
- **7-Zip Probe Harness:** Tests detection of 7-Zip binaries across directories, executables, and PATH variables without executing live external installations.
- **Queue Test Double:** Exercises delete and progress callback semantics in memory.
- **SQLite In-Memory Migrations:** Every database test operates on a fresh temporary SQLite database with all migrations applied.

---

## ⏱️ Performance Budgets

| Operation | Budget | Target |
| --- | --- | --- |
| Cold application startup | < 2s | Native Rust binary initialization |
| Catalog listing query | < 300ms | Indexed SQLite queries with WAL mode |
| Search filter response | < 200ms | Instant client-side or indexed search |
| HTML archive page parse | < 400ms | Streaming scraper |
| Network request fan-out | ~ 1 req/s | Sequential rate-limited scheduler |

---

## 🔗 Related Pages

- [Architecture](Architecture)
- [Installing & Building](Installing-and-Building)
- [Development](Development)
- [Release Process](Release-Process)