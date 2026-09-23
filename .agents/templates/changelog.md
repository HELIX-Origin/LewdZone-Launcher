---
name: changelog
scope: CHANGELOG.md release entries
---

# Changelog Entry (per release)

Add one section _at the top_ of `CHANGELOG.md` per tagged release. Historical
record: every change is listed, newest releases first, living state under
`## Unreleased`. See [Rule 08](../rules/rule-08-release-standards.md).

## Template

```md
# {Title}

{description}

## [{Release Version}]({Release URL})

### {Added | Removed | Fixed | etc}

- **{Commit Message}** [{Commit Hash}]({Commit Link})
- **{Commit Message}** [{Commit Hash}]({Commit Link})

> {separate blank line between grouped bullets — use as needed}

---

| table row | table row | table row |
| :--- | :--- | :--- |
| table item | table description | status icon |
| table item | table description | status icon |

## {Additional section for extra information}

- List item
- List item
```

## Fill rules

1. **`{Release Version}` = `vX.Y.Z`** — exact tag (SemVer, Rule 08).
2. **`{Release URL}`** — the GitHub release URL:
   `https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/vX.Y.Z`.
3. **Category headings** — any of `Added` / `Removed` / `Fixed` / `Changed` /
   `Security` / `Deprecated`. Use those that apply; omit the rest.
4. **Commit entries** — subject mirrors the [commit-message guide](./commit-message-guide.md):
   `**<emoji> <type>(<scope>): <subject>** [(short hash)](full hash URL)`.
   Full hash URL: `https://github.com/HELIX-Origin/LewdZone-Launcher/commit/<full-sha>`.
5. **Summary table** — component / description / status icon (`✅` done, `⏳`
   pending, `⚠️` known issue). Same table, up to date, for every release.
6. **`## Additional`** — anything users need to know that isn't a commit
   (migrations, config changes, breaking notes, upgrade steps).

## Example (Unreleased section)

```md
## Unreleased

### Added

- **docs(repo): add AGENTS operating manual, ROADMAP, TODO, and BUGS trackers** [9673193](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/967319311bae99e61a5965f0723169e7672e5e33)

---

| Component | Description | Status |
| :--- | :--- | :--- |
| Root docs | AGENTS.md, ROADMAP.md, TODO.md, BUGS.md | ✅ |
| Core implementation | Scraping, resolver, download dispatch, CLI engine | ⏳ |
```

## Verify

- [ ] newest release section is at the top, above `Unreleased`
- [ ] every commit since the last release appears under exactly one category
- [ ] release URL matches the tag; hash links point at real commits
- [ ] summary table reflects current project state