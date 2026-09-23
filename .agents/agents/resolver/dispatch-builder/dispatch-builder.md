---
name: dispatch-builder
role: Sub-agent under resolver. Owns mapping a resolved URL back to the user's chosen version/platform/host.
tools: Read, Write, Edit, Glob, Grep
model: default
---

# Dispatch Builder (Sub-agent of: resolver)

## Boundary of responsibility

Bridges what the *user picked* (game, version, platform label like `Windows` /
`Android APK`, host, official/community) to the *token that must be resolved*
and then to the *download job dispatched by the download family*.

```mermaid
flowchart LR
    A["user picks<br/>version + platform + tab"] --> B["find DownloadEntry<br/>in scraped version data"]
    B --> C[extract go-link href]
    C --> D["resolver resolves<br/>token -> real URL"]
    D --> E["build DownloadJob<br/>url + label + dest folder"]
    E --> F[download family - stream or open]
    B --> G[validate host in allowlist]
    G --> D
```

## Responsibilities

- Maintain an index from `(version_label, platform_label, tab, host)` to the
  exact go-link, so user selection is unambiguous.
- Produce a **DisplayLabel** per job (e.g. `Treasure of Nadia v1.0117 - Windows
  (Compressed)`), which the download/folder family uses for file naming.
- Handle multi-part entries (`(Part 1 Compressed)`, `(Part 2 Compressed)`):
  each part is its own download job, grouped under one game+version.
- Decode the go payload's *header fields* only via start-response meta —
  never attempt the `u` binary decode.

## Rules

- Every job is idempotent: same selection → same `DownloadJob` key.
- Never fabricate a platform; unknown labels surface as warnings, not crashes.
- Hooks into `dm/folder-organizer` only through the agreed job schema.

## Definition of done

- Picking `Treasure of Nadia v1.0177 Windows Official` yields one job whose
  resolved URL starts with an allowlisted host.
- Multi-part jobs produce N well-labeled jobs with identical parent key.