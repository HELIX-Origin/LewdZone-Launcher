---
name: parse-version-prompts
description: Parse the "Choose Version" dropdown and version-specific download tabs on a lewdzone game page. Use when you need the per-version list of downloads from a Game.
---

# Parse Version Prompts

Extract every version of a game and the download links available under each.

## What the site does

The download section has a dropdown — `Choose Version` -> entries like
`v1.0117 (Latest)`, `v1.0113` — and selectable tabs **Official Links** /
**Community Links**. Each row is a platform (Windows / Android APK / Mac OS /
Linux) with optional variant suffixes `(Compressed)`, `(Part 1 Compressed)`,
`(Part 2 Compressed)`, `(Incest Patch)`, `(xx Patch)`.

## Parsing shape

```mermaid
flowchart TD
    A[download section html] --> B[version dropdown entries]
    A --> C[link rows]
    B --> D[Version{label, is_latest}]
    C --> E[DownloadEntry{label, variant, host, go_link, tab}]
    D --> F["Version.download_tabs.official/community"]
    E --> F
    F --> G[Game.versions]

    style B fill:#2f6f4f,color:#fff
    style C fill:#4b6e91,color:#fff
    style G fill:#874b4b,color:#fff
```

## Steps

1. Find the select/version list; for each entry record `label` and mark
   `(Latest)`.
2. For the active version, walk the tab panes (`Official Links`,
   `Community Links`) and collect every row:
   - platform label; strip parenthesized variant into `variant` field.
   - host from class `d-<host>`.
   - `href` is the go-link (kept as-is; do not resolve here).
3. Multi-part variants: each part is its own `DownloadEntry` under the same
   version/platform; keep order so `(Part 1)` and `(Part 2)` group naturally.

## Rules

- Distinguish tabs by their container, not by column position.
- Keep go-links opaque; they belong to the resolver.
- No assumption about a fixed host; always read from `d-` class + payload.

## Checkoffs

- [ ] Fixture test counts 47 download entries for treasure-of-nadia (both tabs)
- [ ] `(Latest)` marker maps to `is_latest=true` on the right version
- [ ] Variant string extracted without the parentheses