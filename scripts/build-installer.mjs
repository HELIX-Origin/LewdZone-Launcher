import { execSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readFileSync, chmodSync } from "node:fs";
import { join, resolve } from "node:path";

const pkg = JSON.parse(readFileSync("package.json", "utf-8"));
const version = pkg.version || "0.2.1";

console.log(`\n======================================================`);
console.log(`🚀 Building LewdZone Modern Unified Installer v${version}`);
console.log(`======================================================\n`);

try {
  // 1. Build the Tauri release binary without NSIS/MSI bundling
  console.log("Step 1/2: Compiling release binary with embedded assets...");
  execSync("npx tauri build --no-bundle", { stdio: "inherit" });

  // 2. Output directory for installers
  const outDir = resolve("dist/installer");
  mkdirSync(outDir, { recursive: true });

  const isWindows = process.platform === "win32";
  const isMac = process.platform === "darwin";

  console.log("\nStep 2/2: Packaging standalone unified installer...");

  if (isWindows) {
    const srcExe = resolve("src-tauri/target/release/lewdzone.exe");
    if (!existsSync(srcExe)) {
      throw new Error(`Compiled executable not found at: ${srcExe}`);
    }

    const versionedInstaller = join(outDir, `LewdZone-Setup-${version}.exe`);
    const genericInstaller = join(outDir, "LewdZone-Setup.exe");

    copyFileSync(srcExe, versionedInstaller);
    copyFileSync(srcExe, genericInstaller);

    console.log(`\n✅ Custom Unified Installer successfully generated!`);
    console.log(`   📂 ${versionedInstaller}`);
    console.log(`   📂 ${genericInstaller}`);
    console.log(`\n👉 When launched, it automatically presents the custom tabbed Installer Wizard.\n`);
  } else {
    const srcBin = resolve("src-tauri/target/release/lewdzone");
    if (!existsSync(srcBin)) {
      throw new Error(`Compiled executable not found at: ${srcBin}`);
    }

    const versionedInstaller = join(outDir, `LewdZone-Setup-${version}`);
    const genericInstaller = join(outDir, "LewdZone-Setup");

    copyFileSync(srcBin, versionedInstaller);
    copyFileSync(srcBin, genericInstaller);
    chmodSync(versionedInstaller, 0o755);
    chmodSync(genericInstaller, 0o755);

    console.log(`\n✅ Custom Unified Installer successfully generated!`);
    console.log(`   📂 ${versionedInstaller}`);
    console.log(`   📂 ${genericInstaller}\n`);
  }
} catch (err) {
  console.error("\n❌ Failed to build custom unified installer:", err.message);
  process.exit(1);
}
