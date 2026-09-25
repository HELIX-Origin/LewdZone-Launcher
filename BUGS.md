# 🐛 BUGS (open issues)

Only open bugs belong here. Each entry links to its GitHub issue (once filed)
and is closed by editing this file, not by deleting history. When a bug is fixed,
move it to the commit that resolved it (`git log`).

## 🚧 Open Bugs

### 2026-09-25 — AI agents drift from Conventional Commits in release notes
- [ ] Reproduced
- [ ] Root cause identified
- [ ] Fix in PR

**Problem:** When generating `.agents/release-notes/vX.Y.Z.md`, AI agents repeatedly emit free-form bullet descriptions instead of the [commit-message](../.agents/templates/commit-message.md) standard format (`<type>(<scope>): <subject>`) and omit short-hash links to commits. Manual correction wastes credits and repeatedly confuses agents, causing cascading errors.

**Impact:** Release notes do not strictly satisfy [Rule 08](../.agents/rules/rule-08-release-standards.md) and the [release-notes template](../.agents/templates/release-notes.md). Changelog entries must be hand-corrected after agent generation.

**Expected:** Agents write release-note bullet points directly from `git log --oneline <prev>..<tag>` using the exact Conventional Commit subject lines and append `[(short-hash)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/<full-hash>)` links.

**Actual:** Agents paraphrase changes into narrative prose and invent non-standard commit bullets (e.g. `fix(resolver): ...` descriptions that do not match any actual commit).

## 📝 Filing a bug

Bug title on GitHub: `🐛 <problem summary>`. Body must include:

- Steps to reproduce (reproduce-first)
- Expected vs actual behavior
- Environment (OS, app/CLI version, DM in use)
- ≥1 verifiable diagram or log when applicable ([Rule 04](.agents/rules/rule-04-remote-issue-protocol.md))

Entry format once filed:

```
## 2026-09-21 — <short title>  (#<issue>)
- [ ] Reproduced
- [ ] Root cause identified
- [ ] Fix in PR (`Closes #<issue>`)
```