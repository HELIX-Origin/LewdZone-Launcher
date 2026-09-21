---
name: shortcut-builder
role: Sub-agent under shortcuts. Owns native shortcut creation (.lnk/.desktop/.app), start-menu/Applications placement, and idempotent rebuilds.
tools: Read, Write, Edit, Glob, Grep, Bash
model: default
---

# Shortcut Builder (Sub-agent of: shortcuts)

## Boundary of responsibility

Take an organized game folder + an optional `.ico` and produce working native
shortcuts. This is the ONLY agent allowed to manipulate shortcut files.

## Where shortcuts go

```mermaid
flowchart LR
    A[game exe path] --> B[target verified exists]
    A --> C["workdir = game folder"]
    D[ico path] --> E{icon supplied?}
    E -- yes --> F[assign icon]
    E -- no --> G[generic icon]
    B --> H[build shell object .lnk]
    F --> H
    G --> H
    H --> I[Start Menu lnk]
    H --> J[Desktop lnk]
    H --> K["optional: user-set custom dir"]
    I --> L["db.shortcuts"]
    J --> L
    K --> L

    style H fill:#4b6e91,color:#fff
    style I fill:#2f6f4f,color:#fff
    style J fill:#2f6f4f,color:#fff
    style L fill:#874b4b,color:#fff
```

## Implementation notes (per OS)

- Windows: use the Shell COM interface (`WScript.Shell`) via the `windows`
  crate — no Python, no `win32com`.
  `shell.CreateShortcut(path)` -> set `TargetPath`, `WorkingDirectory`,
  `Arguments`, `IconLocation`, `Description`, then `.Save()`.
- Start Menu root:
  `%APPDATA%\Microsoft\Windows\Start Menu\Programs\<lewdzone>/<Title>.lnk`
  (create a `lewdzone` subfolder by default).
- Desktop root: `%USERPROFILE%\Desktop`.
- Linux: write a `.desktop` entry (`Exec=`, `Icon=`, `Path=`,
  `Terminal=false`) into `~/.local/share/applications/` and optional Desktop.
- macOS: create a Finder alias to the executable via `NSURL`/`FileManager`;
  place it in `~/Applications` and/or Desktop.
- Game names may contain characters invalid in filenames — sanitize per
  `.agents/rules/rule-02-naming-conventions.md` (strip `/<>\|:*?"`).

## Primary-exe detection

```mermaid
flowchart TD
    A[game folder] --> B[glob *.exe at root]
    B --> C{exactly one?}
    C -- yes --> D[use it]
    C -- no --> E{score candidates<br/>name contains game title?}
    E -- yes --> F[prefer title match]
    E -- no --> G[pick first by size; record ambiguity]
```

## Rules

1. Verify `TargetPath` exists at build time; if missing, record a
   `pending-repair` row and still write the `.lnk` (so the user sees what's
   broken).
2. Rebuild = delete existing `.lnk` for the same slot then recreate — never
   duplicate.
3. A `.lnk` whose target moved must be rebuildable from `db.shortcuts` stored
   fields; store `exe_path`, `folder`, `display_name`, `icon_path` at build.
4. If the icon is `None`, use the base game exe's embedded icon if it exists,
   else a generic shell icon.

## Definition of done

- `create_shortcuts(build: ShortcutSpec) -> list[Path]` creates Start Menu +
  Desktop links; second run produces identical results (idempotent).
- Broken-target path covered by unit test (nonexistent exe still yields a
  `.lnk` + repair row).
- Per-OS modules behind `#[cfg(target_os = ...)]`; tests guard so
  non-matching platforms skip.