# 🎨 Theme Development & Styling Guide

> Links between wiki pages are relative and omit the `.md` extension.

LewdZone Launcher features a dynamic runtime theming engine. Themes (skins) allow full customization of the launcher's visual appearance through CSS custom property tokens, requiring zero application restarts.

---

## 📦 Theme Package Structure

Themes reside in the user-accessible skins directory as individual folders containing a `theme.json` manifest:

```text
<skins_root>/
└── ThemeName/
    ├── theme.json            # Required theme definition
    └── assets/               # Optional directory for artwork, icons, or banners
        ├── icon.png
        └── background.png
```

The launcher ships with three built-in reference themes seeded on first launch:
- **Nord:** Arctic, north-bluish palette.
- **Dracula:** Dark, vibrant gothic aesthetic.
- **Material:** Clean, modern material surface design.

---

## 📁 Where Themes Live

| OS | Directory Path |
| --- | --- |
| **Windows** | `<install_dir>\skins\` (alongside the executable) |
| **Linux** | `$XDG_DATA_HOME/lewdzone/skins/` (`~/.local/share/lewdzone/skins/`) |
| **macOS** | `~/Library/Application Support/lewdzone/skins/` |

---

## 📝 The `theme.json` Manifest

A valid `theme.json` requires a `name` and optional `tokens` map:

```json
{
  "name": "Neon Cyberpunk",
  "version": "1.0.0",
  "author": "Community Creator",
  "description": "High contrast cyan and magenta palette",
  "tokens": {
    "--lz-accent": "#FF007F",
    "--lz-primary": "#00F0FF",
    "--lz-cyan": "#00FFFF",
    "--lz-bg": "#05050A",
    "--lz-surface": "#0C0D14",
    "--lz-surface-2": "#161824",
    "--lz-text": "#F0F4F8",
    "--lz-text-dim": "#8A99AD",
    "--lz-danger": "#FF2A6D",
    "--lz-ok": "#05FFA1",
    "--lz-radius": "6px",
    "--lz-gap": "14px",
    "--lz-glow": "0 0 16px rgba(0, 240, 255, 0.4)"
  }
}
```

---

## 🎨 Design Tokens Reference

| CSS Token | Default | Usage |
| --- | --- | --- |
| `--lz-accent` | `#FF4EC8` | Primary accent color, focus rings, hover indicators. |
| `--lz-primary` | `#FF5FB2` | Primary buttons and key interactive elements. |
| `--lz-cyan` | `#22D3EE` | Active navigation highlights, download metrics. |
| `--lz-bg` | `#0A1118` | Base background color for main views and window. |
| `--lz-surface` | `#0E1B26` | Card background and primary panel surfaces. |
| `--lz-surface-2` | `#122A3A` | Raised surfaces, inputs, modal containers. |
| `--lz-text` | `#E8F1F8` | Primary text color. |
| `--lz-text-dim` | `#9AAEC0` | Secondary and descriptive label text. |
| `--lz-danger` | `#FF3B6B` | Destructive actions, cancellation buttons, error badges. |
| `--lz-ok` | `#3DFFA2` | Success status, completed download badges. |
| `--lz-radius` | `4px` | Border radius for cards, inputs, and buttons. |
| `--lz-gap` | `12px` | Grid gap and layout spacing scalar. |
| `--lz-glow` | `0 0 14px rgba(34, 211, 238, 0.35)` | Neon box-shadow glow effects. |

---

## 🛡️ Theme Security

- Themes are strictly declarative JSON data.
- Executable scripts, JavaScript code, or binary files are never executed from theme packages ([Security](Security)).
- Any unknown token keys or malformed CSS values are safely dropped, and corrupt packages fall back gracefully to the built-in default theme.

---

## 🔗 Related Pages

- [Configuration Reference](Configuration)
- [Architecture](Architecture)
- [Security Architecture](Security)
- [Troubleshooting](Troubleshooting)