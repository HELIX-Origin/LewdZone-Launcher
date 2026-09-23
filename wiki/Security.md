# 🔐 Security

> Links between wiki pages are relative and omit the `.md` extension.

## 🧾 Principles

1. **No secrets in the repo.** SteamGridDB, IGDB, updater/signing keys live in
   `~/.config/lewdzone/` or environment variables, never committed. API keys are
   set via the app's Settings → API keys (`settings set <key> --secret`) and
   are never echoed back or logged.
2. **Downloads stay on allowlisted hosts, and never detour.** The launcher
   streams direct-file hosts and hands other resolved URLs to the OS default
   handler — but only for **allowlisted hosts** derived from the site's own
   go-link host table (reverified regularly). Unknown host → refuse. When a
   direct stream is redirected, the redirect must remain on the same host (or a
   dot-boundary subdomain) — any other target is refused.
3. **No shell interpolation** — external processes are spawned with
   `std::process::Command` using argv arrays (`CREATE_NO_WINDOW` on Windows,
   detached session on POSIX). Never build a command string from user input.
4. **Parameterized SQL only** — no string-concatenated SQL. SQLite opened with
   `foreign_keys=ON`; single writer connection.
5. **Path hardening** — all paths via `std::path::PathBuf`, reject traversal
   (`..`), sanitize `\ : * ? " < > |` in generated file names.
6. **Secret redaction in logs** — never log keys/tokens; add a redaction
   filter at the logging boundary.
7. **Resolved URLs are ephemeral** — stored tokens in the DB, never URLs.

## 🛡️ Threat checklist (review gate)

- SQLite injection?
- Path traversal / unsafe filenames?
- URL smuggling via a non-allowlisted host?
- Redirect detour to a foreign host during an in-app stream?
- OS-default-handler misuse (opening a non-URL or a non-allowlisted URL)?
- Secret leakage (keys/tokens in logs, outputs, or the repo)?
- Unsafe redirect following?

Audited by [security-auditor](../.agents/agents/review/security-auditor/security-auditor).
Tooling in the gate: **cargo audit** + **cargo deny** for dependency/CRATE
checking. Full rule: [Rule 10](../.agents/rules/rule-10-security).

## ✅ Allowlist source

Host slugs come from the site's go.js ICONS list (e.g. `mediafire`, `mega`,
`google`, `pixeldrain`, …). The resolved-URL allowlist is derived from that list
and kept in sync during [Release Process](Release-Process); changes require a
security review ADR. Dead services are removed — currently `gofile` is out of
the allowlist. Direct-stream host class is a subset of it (`fileknot` today);
only that subset is ever streamed in-app, and its redirects are validated to
stay on-host.