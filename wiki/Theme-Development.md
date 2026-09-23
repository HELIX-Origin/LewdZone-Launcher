# 🎨 Theme Development

> Links between wiki pages are relative and omit the `.md` extension.

This is the complete guide for creating themes (skins) for the LewdZone
Launcher. Themes color every view through design tokens — a skin never ships
code, only values + assets ([Security](Security)).

## 📦 What a theme is

A theme is a **folder** placed inside the app's user skins folder, with one
subfolder per theme. Embedded theme resources (artwork, icons, …) live **inside
the theme folder, right next to `theme.json`** — with subfolders used freely for
organization:

```
<skins>/<Theme Name>/
├── theme.json          # required manifest (name, version, author, tokens)
└── assets/             # embedded resources (artwork, icons) — inside the theme
    ├── banner.png      #   folder next to theme.json; subfolders are fine
    └── icons/cover.png
```

The **built-in default theme** is compiled into the app and is never stored in
this folder; it stays as the fallback. When you apply a custom theme, its
tokens load in place of the defaults at runtime — no restart needed.

## 🧭 Bundled reference themes

Three working themes ship with every build and become live folders in the skins
folder on first run — **Nord**, **Dracula**, and **Material**. They are the
canonical examples for theme creators: copy one, rename its folder, and edit
the tokens. See the shipped `theme.json` files in
[`skins/`](https://github.com/helix-origin/lewdzone-launcher/tree/main/skins)
for the exact structure.

## 📁 Where themes live per OS

The skins folder is a **user-accessible** location (one subfolder per theme),
chosen per OS so users can always reach their themes:

| OS | Skins folder |
| --- | --- |
| Windows | `<install dir>\skins\` (next to the executable) |
| macOS | `~/Library/Application Support/lewdzone/skins/` (the `.app` bundle is not user-accessible) |
| Linux | `$XDG_DATA_HOME/lewdzone/skins/` (usually `~/.local/share/lewdzone/skins/`) |

Each theme must be its **own subfolder**, named exactly as it should appear in
the **Settings → Appearance** dropdown.

## 📝 Writing `theme.json`

The manifest is small, JSON, UTF-8:

```json
{
  "name": "Pink Neon",
  "version": "1.0.0",
  "author": "you",
  "tokens": {
    "--lz-accent": "#FF4EC8",
    "--lz-bg": "#0A1118"
  }
}
```

### Manifest rules

| Field | Required | Rules |
| --- | --- | --- |
| `name` | yes | must equal the folder name (trimmed, no surrounding whitespace) |
| `version` | no | free-form string (recommend semver) |
| `author` | no | display text |
| `tokens` | no | map of `--lz-*` custom properties; empty = pure default theme |

A manifest that fails validation (e.g. a token key that does not start with
`--lz-`) causes the skin to be **skipped** — the app falls back to the default
theme. Invalid keys inside an otherwise-valid skin are **dropped**, never
applied.

## 🎨 The design tokens

Skins override CSS custom properties. All tokens shown below are the defaults;
set any of them in your `tokens` map to override. They apply app-wide.

| Token | Default | Meaning |
| --- | --- | --- |
| `--lz-accent` | `#FF4EC8` | neon magenta accent (focus, hovers) |
| `--lz-primary` | `#FF5FB2` | hot pink primary |
| `--lz-cyan` | `#22D3EE` | neon cyan (active nav glow) |
| `--lz-bg` | `#0A1118` | deep dark cyan/charcoal page bg |
| `--lz-surface` | `#0E1B26` | panel surface |
| `--lz-surface-2` | `#122A3A` | raised surface (inputs, cards) |
| `--lz-text` | `#E8F1F8` | primary text |
| `--lz-text-dim` | `#9AAEC0` | secondary text |
| `--lz-danger` | `#FF3B6B` | errors / destructive |
| `--lz-ok` | `#3DFFA2` | success states |
| `--lz-radius` | `4px` | corner rounding |
| `--lz-gap` | `12px` | layout spacing scale |
| `--lz-gradient` | `linear-gradient(160deg, #0A1118 0%, #0E1B26 100%)` | page backdrop |
| `--lz-glow` | `0 0 14px rgba(34, 211, 238, 0.35)` | neon glow under accents |
| `--lz-glass` | `rgba(14, 27, 38, 0.55)` | glassmorphism fill (title bar, menus) |

> **Parity rule:** the defaults above must match `src/lib/theme/default.css`
> exactly — both are the source for `--lz-*` values and stay in sync.

## 🖼️ Theme assets

Embedded resources — artwork, icons, and any other files a theme needs — live
**inside the theme folder, right next to `theme.json`**. Use subfolders freely
for organization (e.g. `assets/`, `assets/icons/`). The app resolves an asset
strictly with a sandbox check — a `relative` path must stay within the theme
folder, or it resolves to `None`. The bundled Nord/Dracula/Material themes each
include an `assets/` subfolder as an example. The token layer is the primary
surface today; assets are authored and referenced by views going forward.

## 🧪 Trying a theme

1. Create `<skins>/Pink Neon/theme.json` (folder name + manifest name match);
   the skins folder is the per-OS path shown in the table above.
2. Open **Settings → Appearance**, pick **Pink Neon**.
3. It applies instantly. Switch back to `(default)` any time.
4. Or set it from the CLI: `lewdzone settings set theme "Pink Neon"`.

## 👀 How the app sees skins

- **Settings → Appearance** lists every folder under the skins folder that has
  a valid `theme.json` (sorted, default theme excluded from the list but always
  available as `(default)`). The bundled themes are seeded on first run.
- Applying a theme writes the `theme` setting key and returns the merged token
  set to the webview, which updates CSS variables live (`src/lib/theme/apply`).
- The merge is **defaults overridden by skin tokens** (`effective_tokens` in
  `core/skins.rs`); unknown/default tokens remain when a skin omits them.

## 🔒 Security boundaries

- Skins carry **tokens + assets only**. No scripts, no arbitrary CSS, no
  network calls — the manifest schema only accepts `--lz-*` string values
  ([Rule 10](../.agents/rules/rule-10-security.md)).
- A token whose key fails the `--lz-*` check is dropped, never applied.
- Asset lookup is sandboxed to the theme folder.
- The allowlist and download/resolver paths are never reachable from a skin.

See also: [Configuration](Configuration) → Theme skins, [Design Conventions](Design-Conventions).