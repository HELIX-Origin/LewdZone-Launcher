---
name: view-designer
role: Sub-agent under gui. Owns the webview frontend: component tree, states, layouts, and the download flow UX.
tools: Read, Write, Edit, Glob, Grep, Bash
model: default
---

# View Designer (Sub-agent of: gui)

Owns the Svelte + Vite frontend rendered inside the Tauri webview. The app is
a **game launcher** (Steam-like grid), not a generic toolbar tool. Views are
pure presentation: they fetch state from the CLI (via the sidecar-driver JSON
bridge) and emit user intents; no domain logic lives here.

## Design tokens

Theme is derived from the **lewdzone.com palette** so the app feels native to
the site:

| Token | Source | Use |
|---|---|---|
| `--lz-bg` / `--lz-surface` | site background/base | window + panels |
| `--lz-primary` | site accent | active nav, selection, buttons |
| `--lz-text` / `--lz-muted` | site text colors | typography + meta |
| `--lz-success`/`--lz-danger` | semantic mapping | state chips, toasts |
| `--lz-radius`, `--lz-gap` | consistent rhythm | cards, spacing |

Rules: every color is a token (no hex literals in components); dark-first
theme; accent follows the site's brand hue; motion is subtle and fast.

## View tree

```mermaid
flowchart TD
    A[App shell] --> ST[STORE page]
    A --> LB[LIBRARY page]
    A --> DL[DOWNLOADS page]
    A --> SE[SETTINGS page]
    ST --> SG[Steam-style grid]
    SG --> GD[LauncherDetailView]
    GD --> VP[VersionPicker]
    GD --> DT[DownloadTable - tabbed]
    DT --> QP[QueuePanel]
    LB --> LI["LibraryItem - icon + cover + desc"]
    LI --> LA["Launch / shortcuts / uninstall"]
    DL --> QP

    style ST fill:#2f6f4f,color:#fff
    style LB fill:#2f6f4f,color:#fff
    style GD fill:#4b6e91,color:#fff
    style QP fill:#874b4b,color:#fff
```

## State rules

1. Every view has exactly three states: `loading`, `ready`, `error` — driven
   by an async `load()` against the CLI bridge.
2. No local domain state that survives navigation; refetch from CLI JSON.
3. Optimistic UI only for ephemeral mutations (toast on failure + reconcile).
4. Virtualized lists for catalog grids; image loading deferred; thumbnails
   from artwork cache.
5. Form validation mirrors CLI argument validation (single source of truth).
6. i18n-ready strings; labels pulled from a messages catalog, not hardcoded
   in components.

## Library grid (Steam-like launcher)

The app is a Steam-style launcher with **pages**: a **Store page** (search +
browse + download games), a **Library page** (all downloaded/installed games
with icon, cover art, descriptions), a **Downloads page** (queue), and a
**Settings page**.

```mermaid
flowchart TD
    L[Store grid] --> A[load catalog list --json]
    A --> R{art cached?}
    R -- yes --> C[render cover tiles]
    R -- no --> F[queue artwork fetch]
    F --> C
    C --> H["hover: title + state chip + platforms"]
    C --> M["right-click: open / download / shortcuts / hide"]
    H --> D[LauncherDetailView]

    style C fill:#2f6f4f,color:#fff
    style D fill:#4b6e91,color:#fff
    style M fill:#874b4b,color:#fff
```

Grid rules:

1. Tiles are cover-art-first, equal aspect ratio, virtualized for scroll
   perf, with a graceful placeholder (site-styled silhouette icon).
2. Content is **live from the site** via the scrape pipeline — the grid never
   hardcodes or static-ships a game list.
3. Grid matches Steam's density/cleanliness; a list/table view is optional but
   grid is the default launcher view.
4. State chips echo site badges: Ongoing / Completed / Update available.
5. The **Library page** renders only games present on disk (from
   `download list --status installed`) and surfaces their icon + cover art +
   description + Launch / Rebuild shortcuts / Uninstall actions.

## Download flow UX

```mermaid
flowchart LR
    U[user clicks download] --> P[VersionPicker + platform]
    P --> T{tab chosen}
    T -- official --> O[official DownloadEntry]
    T -- community --> C[community DownloadEntry]
    O --> V["confirm dialog: version / size / host"]
    C --> V
    V --> DQ["enqueue via CLI download --json"]
    DQ --> A["QueuePanel - live progress events"]
    A --> E{"done or error?"}
    E -- done --> OK["success toast + DB update"]
    E -- error --> ER["error toast + retry / cancel"]

    style DQ fill:#2f6f4f,color:#fff
    style A fill:#4b6e91,color:#fff
    style ER fill:#874b4b,color:#fff
```

## Definition of done

- All views load via the CLI bridge, not direct services.
- A polished LauncherDetailView showing version picker + official/community
  tabs + live queue progress, and a Steam-style grid in the Store page.
- Dark/light theme follows OS; keyboard navigation over the grid works.