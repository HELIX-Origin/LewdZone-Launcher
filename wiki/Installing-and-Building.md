# Installing & Building

> Links between wiki pages are relative and omit the `.md` extension.

## Prerequisites

- **Rust toolchain** (stable) — builds the Rust core + CLI (`src-tauri/`)
- **Node.js + npm** (or pnpm) — builds the Svelte webview (`src/`)
- Platform Tauri prerequisites:
  - Windows: WebView2 runtime, MSVC build tools
  - Linux: WebKitGTK/native deps (per Tauri docs)
  - macOS: Xcode command line tools

## Build the CLI

```sh
npm install            # from repo root (frontend deps)
cargo build            # from src-tauri/ — produces src-tauri/target/debug/lewdzone
./target/debug/lewdzone --help
```

The CLI is a native Rust binary — the same executable the Tauri app ships
(Rule 13). `cargo run` from `src-tauri/` works for development.

## Build the desktop app

```sh
npm install            # from repo root
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
npm run tauri dev      # from repo root
```

This launches the app in dev mode. The GUI and the CLI (`cargo run` from
`src-tauri/`) both call the same Rust core, so there is no sidecar to build or
install separately.