# Installing & Building

> Links between wiki pages are relative and omit the `.md` extension.

## Prerequisites

- **Python 3.x** + `pip` (or `uv`)
- **Rust toolchain** (stable) for the Tauri shell
- **Node.js + npm** (or pnpm) for the Svelte webview
- Platform Tauri prerequisites:
  - Windows: WebView2 runtime, MSVC build tools
  - Linux: WebKitGTK/native deps (per Tauri docs)
  - macOS: Xcode command line tools

## Build the CLI

```sh
pip install -e .
lewdzone-launcher --help
```

The CLI is pure-ish Python (stdlib-first; Pillow for icon conversion). It has
no build step beyond an installable package in `src/`.

## Package the CLI as a sidecar

The desktop app embeds the CLI as a **PyInstaller** executable
(`externalBin`). Build it first so `tauri build` can bundle it:

```sh
pyinstaller --onefile --name lewdzone-launcher \
  src/lewdzone_launcher/__main__.py
```

The sidecar version must equal the app version (see
[Release Process](Release-Process)).

## Build the desktop app

```sh
cd desktop
npm install
npm run tauri build
```

Artifacts per platform (see [Architecture](Architecture) → Packaging):

| Platform | Formats |
| --- | --- |
| Windows | NSIS installer, MSI |
| macOS | `.app` bundle + DMG |
| Linux | AppImage, deb, rpm |

- **Signing:** Windows Authenticode via `signingIdentities`; macOS app
  notarization. Signing keys are CI secrets.
- **Updates:** `@tauri-apps/plugin-updater` with channel keys (CI secrets).

## Development mode

```sh
cd desktop
npm run tauri dev
```

This launches the app pointed at the **source** sidecar (requires the CLI
installed on `PATH`). Production mode resolves the bundled exe next to the app
binary and verifies its version before spawning.