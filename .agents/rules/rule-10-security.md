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

1. **No secrets in the repo.** Never commit SteamGridDB API keys, tokens, or
   `.env` files. Keys load from `~/.config/lewdzone/` or env at runtime.
2. **Resolved-URL allowlist.** Only hosts present in the go.js `ICONS` list
   (or a re-verified allowlist) may be handed to a download manager. Anything
   unknown → refuse and log (Rule 07).
3. **Parameterized SQL always.** No string-concatenated SQL (Rule 06).
4. **`std::process::Command` argv arrays** always; no user input interpolated
   into a shell string (Rule 07). Windows: `CREATE_NO_WINDOW`; POSIX: detached
   session.
5. **Paths are user-relative, sanitized.** Download root is configurable but
   resolved against the user's home; game folder names sanitize
   `: * ? " < > |` (Rule 02) before join; reject traversal (`..`).
6. **Do not follow unknown redirects.** Only trusted hosts redirected-to are
   followed (scraper sees `lewdzone.com` + `www.steamgriddb.com`).
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
    A --> E["download-manager process spawn"]
    A --> F["secret leakage"]
    B --> B1["parameterized sql + schema"]
    C --> C1["sanitize + reject traversal"]
    D --> D1["allowlist + refuse unknown"]
    E --> E1["argv array + no shell"]
    F --> F1["no secrets, redact logs"]
```

## Audit commands

- `cargo audit` (known-vulnerability scan of the dependency tree)
- `cargo deny check` (licenses + advisories + bans)

## Acceptable-risk callouts

- The Tauri webview serves only bundled Svelte code; no remote content is
  loaded into it.
- Shortcut creation uses the platform-native mechanism: the native Windows API
  for `.lnk` files in the user's own Start Menu/Desktop folders; `.desktop`
  files on Linux; `.app`/alias on macOS.