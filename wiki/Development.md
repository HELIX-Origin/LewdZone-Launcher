# 🛠️ Development & Engineering Guide

> Links between wiki pages are relative and omit the `.md` extension.

This guide provides an overview of repository structure, daily engineering workflows, and verification requirements for contributing to the LewdZone Launcher.

---

## 📁 Repository Layout

```text
lewdzone-launcher/
├── .agents/                      # Agent specifications, rules, templates, and ADRs
│   ├── adr/                      # Architecture Decision Records
│   ├── rules/                    # Enforceable repository rules (Rule 00 - 13)
│   └── templates/                # Issue, agent, and module scaffolds
├── src/                          # Tauri webview frontend (Svelte 5 + Vite + TS)
│   ├── lib/                      # Components, theme engine, and test suites
│   └── routes/                   # Application views (Store, Library, Downloads, Settings)
├── src-tauri/                    # Rust core backend & CLI application
│   ├── src/
│   │   ├── core/                 # Shared core logic (download, extract, queue, folder)
│   │   ├── db/                   # SQLite database models and migrations
│   │   ├── scraper/              # LewdZone HTML parsers and HTTP fetcher
│   │   ├── resolver.rs           # Token resolution & host allowlist verification
│   │   ├── cli.rs                # Native Rust command-line interface
│   │   └── lib.rs                # Tauri entry point, IPC commands, and tray setup
│   └── Cargo.toml                # Rust crate dependencies and metadata
└── wiki/                         # GitHub wiki documentation suite
```

---

## 🤝 Daily Development Commands

```bash
# Start desktop app with hot module reloading
npm run tauri dev

# Run CLI commands directly in development
cargo run --manifest-path src-tauri/Cargo.toml -- --help
cargo run --manifest-path src-tauri/Cargo.toml -- list --library

# Format and lint codebase
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

# Execute test suites
cargo test --manifest-path src-tauri/Cargo.toml
npm run test
npm run check
```

---

## 🏛️ Contribution Rules & Hygiene

1. **One Core, Two Entry Points:** Any feature added to the GUI must have an equivalent CLI command backed by the same Rust core function.
2. **Hermetic Testing:** Tests must run offline without touching live networks unless explicitly tagged with `#[ignore]`.
3. **No Shell Invocations:** Subprocesses must be spawned using explicit argument lists (`std::process::Command`), never through shell strings ([Security](Security)).
4. **No Secrets in Config:** Passwords and API tokens must be saved to the encrypted SQLite database, not `config.json`.

---

## 🔗 Related Pages

- [Architecture](Architecture)
- [Agent Ecosystem](Agents)
- [Testing & QA](Testing)
- [Release Process](Release-Process)
- [Design Conventions](Design-Conventions)