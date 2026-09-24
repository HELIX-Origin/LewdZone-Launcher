# 🏗️ Installing & Building

> Links between wiki pages are relative and omit the `.md` extension.

LewdZone Launcher is built with **Tauri 2**, utilizing a Rust backend and a modern Svelte 5 frontend. The single compiled executable acts as both the graphical desktop application and the standalone CLI tool (Rule 03, Rule 13).

---

## ✅ Prerequisites

1. **Rust Toolchain:** Stable Rust compiler and Cargo (`rustup update stable`).
2. **Node.js & Package Manager:** Node.js (v18 or higher) and `npm` (or `pnpm`).
3. **OS-Specific Tauri Dependencies:**
   - **Windows:** Microsoft Visual Studio C++ Build Tools and WebView2 Runtime (pre-installed on Windows 10/11).
   - **Linux:** WebKitGTK and development libraries (e.g. `libwebkit2gtk-4.1-dev`, `libssl-dev`, `libayatana-appindicator3-dev`).
   - **macOS:** Xcode Command Line Tools (`xcode-select --install`).
4. **7-Zip Console Executable (Runtime Tool):** Recommended for extracting `.zip`, `.7z`, and `.rar` downloads. See the **[Archive Extraction & 7-Zip Setup Guide](Archive-Extraction)**.

---

## 🔨 Building the CLI and Rust Core

To build only the native Rust binary (which includes both CLI subcommands and Tauri backend logic):

```bash
cd src-tauri
cargo build --release
```

The resulting binary will be created at:
- `src-tauri/target/release/lewdzone.exe` (Windows)
- `src-tauri/target/release/lewdzone` (Linux/macOS)

Test the binary:
```bash
./target/release/lewdzone --version
./target/release/lewdzone --help
```

---

## 💻 Building the Desktop Application

To compile the production desktop application and bundle platform installers:

```bash
# Install frontend dependencies from the repository root
npm install

# Build release bundles
npm run tauri build
```

Generated installer artifacts:

| Operating System | Output Formats |
| --- | --- |
| **Windows** | NSIS installer (`.exe`), MSI installer (`.msi`) |
| **macOS** | Application bundle (`.app`), Apple Disk Image (`.dmg`) |
| **Linux** | AppImage (`.AppImage`), Debian package (`.deb`), RPM (`.rpm`) |

---

## 🧑‍💻 Development Loop

To run the application with live hot-reloading:

```bash
# Start Vite frontend and Tauri development shell
npm run tauri dev
```

Run automated verification checks:

```bash
# Rust core linting and test suite (from src-tauri/)
cargo fmt --check
cargo clippy -- -D warnings
cargo test

# Frontend typecheck and Vitest suite (from repo root)
npm run check
npm run test
```

---

## 🔗 Related Pages

- [Getting Started](Getting-Started)
- [Archive Extraction & 7-Zip Setup](Archive-Extraction)
- [Architecture](Architecture)
- [Testing & QA](Testing)
- [Release Process](Release-Process)