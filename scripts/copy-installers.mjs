// Copy the installers Tauri just produced (src-tauri/target/<profile>/bundle/*)
// into the repo-root build/ folder, which is the designated installer drop.
// Cross-platform: understands every bundle layout Tauri emits.

import { cpSync, existsSync, mkdirSync, readdirSync, statSync } from "node:fs";
import { basename, join } from "node:path";

const root = process.cwd();
const srcRoot = join(root, "src-tauri");

function bundleRoots() {
  const target = join(srcRoot, "target");
  if (!existsSync(target)) return [];
  const out = [];
  for (const profile of ["release", "debug"]) {
    const bundle = join(target, profile, "bundle");
    if (existsSync(bundle)) out.push(bundle);
  }
  return out;
}

function collectFiles(dir, acc = []) {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) collectFiles(p, acc);
    else acc.push(p);
  }
  return acc;
}

const roots = bundleRoots();
if (roots.length === 0) {
  console.error("no src-tauri/target/<profile>/bundle found — did the build run?");
  process.exit(2);
}

const dest = join(root, "build");
mkdirSync(dest, { recursive: true });

let copied = 0;
for (const bundle of roots) {
  for (const file of collectFiles(bundle)) {
    if (!/\.(exe|msi|msix|appx|dmg|pkg|deb|rpm|AppImage|tar\.gz)$/.test(file)) continue;
    cpSync(file, join(dest, basename(file)), { force: true });
    copied += 1;
  }
}

console.log(`copied ${copied} installer(s) to build/`);
if (copied === 0) {
  console.warn("no installer artifacts matched; check bundle targets in tauri.conf.json");
}