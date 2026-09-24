# 🔗 Shortcuts & Playtime Tracking

> Links between wiki pages are relative and omit the `.md` extension.

LewdZone Launcher provides native operating system desktop shortcuts and SQLite-backed gameplay session and playtime tracking. Both features are exposed through the Svelte desktop application and the native Rust CLI (`lewdzone shortcuts` and `lewdzone launch`).

---

## 🖥️ Native Desktop & Start Menu Shortcuts

Shortcuts allow players to launch installed LewdZone titles directly from their desktop or application menu without having to keep the main launcher window open.

### Cross-Platform Architecture

| Operating System | Format | Storage Location | Icon Integration |
| --- | --- | --- | --- |
| **Windows** | `.lnk` | `Desktop` & `Start Menu\Programs\LewdZone Games` | Windows Script Host sets `IconLocation = "{exe_path},0"` (embedded executable icon). |
| **Linux** | `.desktop` | `~/Desktop/` & `~/.local/share/applications/` | FreeDesktop compliant specification with `Exec`, `Path`, and `Terminal=false`. |
| **macOS** | `.command` | `~/Desktop/` | Executable bash wrapper script (`chmod 755`) pointing to target app/binary. |

### CLI Usage

```bash
# Generate desktop and start menu shortcuts for a game:
lewdzone shortcuts treasure-of-nadia

# Explicit game flag:
lewdzone shortcuts --game wild-life
```

### GUI Usage

Inside the **Library** view (`/library`), each installed game card contains a shortcut action button beside the **Launch** button:
1. Hover over any game card.
2. Click the **Shortcut** icon (external link arrow).
3. The launcher invokes `create_shortcut` in the Rust core and displays a toast confirmation: `Created shortcut for <Game Title>`.

---

## ⏱️ Playtime & Session Tracking

Playtime and launch history are stored in SQLite and updated in real time.

### Schema (`game_stats`)

The `game_stats` table tracks launch metrics keyed by the game slug:

```sql
CREATE TABLE IF NOT EXISTS game_stats (
    slug TEXT PRIMARY KEY,
    playtime_seconds INTEGER NOT NULL DEFAULT 0,
    play_count INTEGER NOT NULL DEFAULT 0,
    last_played_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### Process Lifecycle Tracking

When a game is launched (via `lewdzone launch <slug>` or clicking **Launch** in the UI):
1. **Launch Event:** `record_game_launch` is called on the SQLite database, incrementing `play_count` and recording the current UTC ISO timestamp in `last_played_at`.
2. **Execution:** The executable is spawned as a child process using the system's native process runner (`std::process::Command`).
3. **Detached Worker Thread:** A background worker thread is spawned with the child process handle. It waits asynchronously on `child.wait()`.
4. **Session Duration:** When the game window exits or is closed by the player, the worker computes `start_instant.elapsed().as_secs()` and commits the duration to `game_stats` via `repo::add_game_playtime(conn, slug, elapsed_seconds)`.

### Library Sorting & Badges

In the **Library** view, game cards display:
- **Playtime Badge:** `Unplayed`, `Xm`, or `X.Xh` formatted badges.
- **Last Played Timestamp:** Human-readable relative time (`Just now`, `5m ago`, `2h ago`, `Yesterday`, `3d ago`).

Players can sort installed games using the **Sort** dropdown:
- **A – Z:** Alphabetical by title.
- **Recently Played:** Games ordered by `last_played_at` descending.
- **Most Played:** Games ordered by `playtime_seconds` descending.
- **Recently Installed:** Games ordered by `installed_at` timestamp.

---

## 🔗 Related Pages

- [CLI Reference](CLI-Reference)
- [Architecture](Architecture)
- [Getting Started](Getting-Started)
- [Configuration](Configuration)
