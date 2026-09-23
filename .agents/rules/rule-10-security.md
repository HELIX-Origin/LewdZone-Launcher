---
name: security
rule_number: "10"
scope: secrets, urls, sql, process spawn, artifacts
enforcement: security-auditor + cargo audit + cargo deny
---

# Rule 10: Security & Secrets

The threat model is explicit (see
[security-auditor](../agents/review/security-auditor/security-auditor.md)).
Every resolved URL, every path, every spawned process is a trust boundary.

## Hard rules

1. **No secrets in the repo.** Never commit SteamGridDB / IGDB / updater keys,
   tokens, or `.env` files. Content-provider API keys live in the SQLite
   `secret` table (Rule 06) and are surfaced as presence-only in the UI.
2. **Resolved-URL allowlist.** Only hosts present in the go.js `ICONS` list
   (or a re-verified allowlist) may be dispatched — either streamed in-app or
   handed to the OS default handler. Anything unknown → refuse and log
   (Rule 07).
3. **Parameterized SQL always.** No string-concatenated SQL (Rule 06).
4. **`std::process::Command` argv arrays** always; no user input interpolated
   into a shell string. For `open_url` use the platform-native opener without
   a shell: Windows `rundll32 url.dll,FileProtocolHandler`
   (`CREATE_NO_WINDOW`); POSIX `open` / `xdg-open` detached.
5. **Paths are user-relative, sanitized.** Download root is configurable but
   resolved against the user's home; game folder names sanitize
   `: * ? " < > |` (Rule 02) before join; reject traversal (`..`).
6. **Do not follow unknown redirects.** An in-app stream may follow a redirect
   only to the same host or a dot-boundary subdomain, max 3 hops; a cross-host
   detour aborts the stream (Rule 07). The scraper sees `lewdzone.com` plus
   content-provider domains: `www.steamgriddb.com`, `api.igdb.com`,
   `api.twitch.tv`, `www.vndb.org`, `itch.io`, `store.steampowered.com`,
   `api.steampowered.com`, and `www.indiedb.com`.
7. **No secrets in logs** (Rule 12 redaction).
8. **Dependency hygiene:** `cargo audit` and `cargo deny` run in the review
   gate.

## Threat checklist (per build task)

```mermaid
flowchart TD
    A["threat surface"]
    A --> B["sqlite injection"]
    A --> C["path traversal"]
    A --> D["resolved-URL smuggling"]
    A --> E["stream redirect detour"]
    A --> F["secret leakage"]
    B --> B1["parameterized sql + schema"]
    C --> C1["sanitize + reject traversal"]
    D --> D1["allowlist + refuse unknown"]
    E --> E1["same-owner redirects only"]
    F --> F1["no secrets, redact logs"]
```

## Audit commands

- `cargo audit` (known-vulnerability scan of the dependency tree)
- `cargo deny check` (licenses + advisories + bans)

## Content-provider secrets

External enrichment providers (SteamGridDB, IGDB, VNDB, itch.io, Steam,
IndieDB) may require API keys. Those keys follow the same secret handling as
any other launcher secret:

- Keys are stored in the SQLite `secret` table, never in `config.json`.
- A provider whose key is missing is silently skipped; it never blocks the UI.
- Keys are read at runtime and are never echoed in commands, settings output,
  or logs (Rule 12 redaction).
- All provider network calls use the polite `scraper::fetch` agent over HTTPS
  (Rule 05) and respect the same timeout/retry budget.

## Acceptable-risk callouts

- The Tauri webview serves only bundled Svelte code; no remote content is
  loaded into it.
- Shortcut creation uses the platform-native mechanism: the native Windows API
  for `.lnk` files in the user's own Start Menu/Desktop folders; `.desktop`
  files on Linux; `.app`/alias on macOS.