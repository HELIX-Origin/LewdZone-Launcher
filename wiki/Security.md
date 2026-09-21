# Security

> Links between wiki pages are relative and omit the `.md` extension.

## Principles

1. **No secrets in the repo.** SteamGridDB, IGDB, updater/signing keys live in
   `~/.config/lewdzone/` or environment variables, never committed. API keys are
   set via the app's Settings → API keys (`settings set <key> --secret`) and
   are never echoed back or logged.
2. **The tool never downloads content itself** — resolved URLs are handed to a
   download manager, and only for **allowlisted hosts** derived from the site's
   own go-link host table (reverified regularly). Unknown host → refuse.
3. **No shell interpolation** — subprocesses always `shell=False` (Windows:
   `CREATE_NO_WINDOW`; POSIX: detached session). Never build command strings
   from user input.
4. **Parameterized SQL only** — no f-string SQL. SQLite opened with
   `foreign_keys=ON`; single writer connection.
5. **Path hardening** — all paths via `pathlib`, reject traversal (`..`),
   sanitize `\ : * ? " < > |` in generated file names.
6. **Secret redaction in logs** — never log keys/tokens; add a redaction
   filter at the logging boundary.
7. **Resolved URLs are ephemeral** — stored tokens in the DB, never URLs.

## Threat checklist (review gate)

- SQLite injection?
- Path traversal / unsafe filenames?
- URL smuggling via a non-allowlisted host?
- Download-manager spawn injection?
- Secret leakage (keys/tokens in logs, outputs, or the repo)?
- Unsafe redirect following?

Audited by [security-auditor](../.agents/agents/review/security-auditor/security-auditor).
Tooling in the gate: **bandit** + **pip-audit**. Full rule:
[Rule 10](../.agents/rules/rule-10-security).

## Allowlist source

Host slugs come from the site's go.js ICONS list (e.g. `mediafire`, `mega`,
`gofile`, `drive`, …). The resolved-URL allowlist is derived from that list and
kept in sync during [Release Process](Release-Process); changes require a
security review ADR.