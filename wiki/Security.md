# 🔐 Security Architecture & Threat Model

> Links between wiki pages are relative and omit the `.md` extension.

LewdZone Launcher is built to interact with external web services while maintaining zero telemetry, complete credential isolation, and strict process execution boundaries.

---

## 🧾 Core Security Principles

1. **Zero Telemetry:** The application phones home nowhere. No analytics, tracking pixels, crash reporters, or usage beacons are present ([Privacy Policy](../../PRIVACY.md)).
2. **Secrets Never Committed or Exposed:** External API credentials (SteamGridDB, IGDB, etc.) are saved encrypted in the SQLite `secret` table and are never written to `config.json`, logs, or console output.
3. **Blocked Hosts Blacklist:** Instead of a restrictive allowlist, the launcher enforces a blacklist of dead, defunct, or known malicious download hosts (`gofile`, `gofiles`, `zippyshare`, `cdnclick`, `anonfile`, `anonfiles`, `anonzip`, `uptobox`, `yourfilestore`, `qiwi`, `transfersh`). Any functional, unblocked host is permitted, providing broad flexibility for users while protecting against dead or unsafe mirrors.
4. **Same-Host Redirect Validation:** During direct file streaming (`fileknot`), HTTP redirects are monitored. The target must remain on the exact same host or an authorized subdomain. Detours to foreign domains are rejected.
5. **No Shell Interpolation:** All subprocesses (including game execution and the **7-Zip console executable**) are spawned directly via `std::process::Command` with argument vectors and `shell=false`. On Windows, processes run silently with `CREATE_NO_WINDOW` (`0x0800_0000`).
6. **In-App Sandboxed Resolver:** Protected links requiring verification or countdowns are displayed in a sandboxed, isolated webview window that blocks third-party popups, malicious ads, and background clickjacking.
7. **Ephemeral Tokens over URLs:** Database records persist go-link tokens rather than resolved URLs. Resolved URLs are considered single-use and ephemeral.
8. **Path Sanitization & Traversal Defense:** All user and game paths use `std::path::PathBuf`, reject relative traversal sequences (`..`), and sanitize forbidden characters (`\ : * ? " < > |`).
9. **Parameterized Database Queries:** SQLite is opened with WAL mode and `foreign_keys=ON`. All SQL interactions use strictly parameterized statements to eliminate SQL injection risks.

---

## 🛡️ Threat Checklist

- [x] **SQL Injection:** Mitigated via Rusqlite parameterized queries throughout `src-tauri/src/db/`.
- [x] **Path Traversal:** Mitigated via `core::folder::within_root` and filename sanitization.
- [x] **Subprocess Injection:** Mitigated via structured argument arrays in `Command::new` without shell interpreters.
- [x] **Malicious / Dead Hosts:** Mitigated via `resolver::is_blocked_host` rejecting blacklisted hosts.
- [x] **Credential Leakage:** Mitigated via SQLite secret storage and automatic log redaction filters.
- [x] **Adware / Script Hijacking:** Mitigated via dedicated in-app sandboxed webview resolver.

---

## 🔗 Related Pages

- [Downloads & In-App Streaming](Download-Managers)
- [Archive Extraction & 7-Zip Guide](Archive-Extraction)
- [Configuration](Configuration)
- [Release Process](Release-Process)