import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { resolve } from "node:path";

const targetTag = process.argv[2] || process.env.GITHUB_REF_NAME || "";
const cleanVer = targetTag.replace(/^v/, "");

const changelog = readFileSync(resolve("CHANGELOG.md"), "utf-8");

// Match ## [vX.Y.Z] or ## [X.Y.Z]
const regex = new RegExp(`##\\s*\\[v?${cleanVer.replace(/\\./g, "\\.")}\\][^\n]*\n([\\s\\S]*?)(?=\n##\\s*\\[|\n---\\s*\n##|$)`, "i");
const match = changelog.match(regex);

let notes = "";
if (match && match[1]) {
  notes = match[1].trim();
} else {
  notes = `Release ${targetTag}`;
}

mkdirSync(resolve("dist"), { recursive: true });
const outPath = resolve("dist/release-notes.md");
writeFileSync(outPath, notes, "utf-8");
console.log(`Extracted release notes for ${targetTag} to ${outPath}`);
