# Security Policy

## 1. Reporting a Vulnerability

Please **do not disclose security issues publicly** (issues, PRs, forums)
before they are resolved.

- **Preferred:** report via GitHub Security Advisories at
  https://github.com/HELIX-Origin/LewdZone-Launcher/security/advisories
- **Alternative:** open a private report through the repository's security
  tab, or contact the maintainers through the channels linked in the repo.

Please include:

- A description of the vulnerability and its impact.
- Affected version(s).
- Steps to reproduce or a minimal PoC.
- Any suggested fix, if you have one.

## 2. Vulnerability Response Timeline

| Stage | Target |
| --- | --- |
| Acknowledge report | Within 48 hours |
| Initial triage | Within 5 business days |
| Fix + release | Depends on severity; coordinated disclosure where possible |
| Public advisory | After the fix is released |

We practice **coordinated disclosure**: details of a vulnerability are made
public only after a patched release is available, unless public disclosure is
required earlier.

## 3. Architectural Security Principles

- **Zero telemetry.** The application phones home nowhere; no analytics, no
  crash reporting, no beacon on launch.
- **Secrets never committed.** API keys live in git-ignored local config; the
  repository contains no real credentials. Environment-provided values are
  preferred where supported.
- **No shell interpolation.** All external commands (download managers, the
  CLI, and other helper binaries) are spawned with argument arrays and
  `shell=false`; no untrusted input is ever interpolated into a shell string.
- **Component boundaries.** The webview never talks to the site, the database,
  or download managers directly — every action flows through the Rust core,
  which validates and allowlists inputs.
- **Minimal dependencies.** The dependency set is kept deliberately small to
  reduce the attack surface.
- **TLS everywhere.** All outbound HTTP uses HTTPS; certificate verification is
  not disabled.

## 4. Security Best Practices for Host Administrators

- Run the latest release.
- Keep resolved download URLs out of logs and transcripts; they expire anyway
  (tokens, not URLs, are stored).
- Review the local `lewdzone.db` contents and configuration for any data you
  consider sensitive before sharing or backing up.
- Use a dedicated, strictly-compliant allowance for any content you download,
  per the laws of your jurisdiction.