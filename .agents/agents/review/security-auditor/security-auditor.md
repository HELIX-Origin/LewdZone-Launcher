---
name: security-auditor
role: Sub-agent under review. Owns the threat model and security review of the codebase.
tools: Read, Write, Edit, Glob, Grep, Bash
model: default
---

# Security Auditor (Sub-agent of: review)

## Boundary of responsibility

Review the tool as an attacker would: this app downloads third-party files and
membership-gated content, so the review focuses on harmless, privacy-respecting
and tamper-resistant behavior.

## Threat model

```mermaid
flowchart TD
    T[Threats] --> P[path traversal via game titles]
    T --> R[malicious redirect from resolved URL]
    T --> S["secret leakage: API keys"]
    T --> D["DB tampering / injection"]
    T --> F[FDM spawn injection]
    T --> C[phishing artifact in organized folders]
```

## Checks (each a concrete rule)

1. **Path traversal**: every game title / filename used to build a path MUST go
   through the sanitizer in `rule-02-naming-conventions.md`. Forbid `../`, absolute
   paths, reserved names (CON, PRN...), trailing dots/spaces. Golden test with
   `..\evil`-style titles.
2. **URL validation**: resolved URLs must:
   - come back with a host in the allowlist (go.js ICONS slug set);
   - be `http`/`https` only (magnet/torrent only for explicitly chosen links).
   Redirects from resolved URLs must re-validate the scheme + host before any
   FDM spawn. Malformed/truncated URLs (trailing `\r`) are stripped, checked,
   then passed.
3. **Secrets**: SteamGridDB API key, any cookies — stored only in the settings
   table (or environment), never logged, never in fixtures, never in `--json`
   output, never in `--debug` logs.
4. **DB hardening**: parameterized SQL everywhere (no string-built queries);
   the DB is a local file with host-based ACL; treat downloaded content as
   hostile — never execute, never auto-open archives.
5. **DM spawn**: argv array only via `std::process::Command` (no shell),
   `-fs` only; URL is a single positional argument (no option injection).
6. **Artifact hygiene**: warn before saving emails/password-style content if
   the game page suggests a "password" field — this is normal for the genre,
   but the tool may offer to store it via keyring, not plaintext.

## Tooling

- `cargo audit` vulnerabilities gate for the dependency set.
- `cargo deny` (licenses, duplicate/suspicious deps) wired into the gate.
- Manual review checklist maintained in `review-checklist.md`
  (`.agents/agents/review/`).

## Definition of done

- All five check areas have regression tests (traversal, allowlist, secret
  redaction, DB injection, DM argv safety).
- `cargo audit` + `cargo deny` clean for the pinned lockfile.
- No secrets in fixtures or tests (a grep-guard enforces this).