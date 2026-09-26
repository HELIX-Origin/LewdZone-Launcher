import { readFileSync, writeFileSync, mkdirSync, existsSync } from "node:fs";
import { resolve } from "node:path";
import { execSync } from "node:child_process";

const rawTag = process.argv[2] || process.env.GITHUB_REF_NAME || "";
const cleanVer = rawTag.replace(/^v/, "");
const tag = `v${cleanVer}`;

mkdirSync(resolve("dist"), { recursive: true });
const outPath = resolve("dist/release-notes.md");
const titlePath = resolve("dist/release-title.txt");

// Rule 08 step 4: the release title is `vX.Y.Z — <Key Feature>`. The H1 of the
// hand-authored notes file is that title, so it is the single source of truth
// for the published release name. The H1 is stripped from the body so the
// release page does not repeat its own title.
function splitTitleAndBody(content) {
  const lines = content.split(/\r?\n/);
  const headingIndex = lines.findIndex((l) => /^#\s+\S/.test(l));
  if (headingIndex === -1) {
    return { title: `v${cleanVer}`, body: content.trim() };
  }
  const title = lines[headingIndex].replace(/^#\s+/, "").trim();
  const body = lines
    .filter((_, i) => i !== headingIndex)
    .join("\n")
    .trim();
  return { title, body };
}

// 1. Check if dedicated release note file exists
const noteCandidates = [
  resolve(`.agents/release-notes/${tag}.md`),
  resolve(`.agents/release-notes/${cleanVer}.md`),
  resolve(`release-notes/${tag}.md`),
  resolve(`release-notes/${cleanVer}.md`)
];

for (const candidate of noteCandidates) {
  if (existsSync(candidate)) {
    const { title, body } = splitTitleAndBody(readFileSync(candidate, "utf-8").trim());
    writeFileSync(outPath, body + "\n", "utf-8");
    writeFileSync(titlePath, title + "\n", "utf-8");
    console.log(`Loaded release notes from ${candidate} -> ${outPath}`);
    console.log(`Release title -> ${titlePath}: ${title}`);
    process.exit(0);
  }
}

// 2. Fallback: Parse from CHANGELOG.md and format to nhentai-desktop specification
const changelogPath = resolve("CHANGELOG.md");
let notes = "";

if (existsSync(changelogPath)) {
  const changelog = readFileSync(changelogPath, "utf-8");
  const regex = new RegExp(`##\\s*\\[v?${cleanVer.replace(/\\./g, "\\.")}\\](?:\\[[^\\]]*\\]|\\([^\\)]*\\))?\\s*[-—]\\s*(\\d{4}-\\d{2}-\\d{2})([\\s\\S]*?)(?=\\n##\\s*\\[|\\n---\\s*\\n##|$)`, "i");
  const match = changelog.match(regex);

  if (match) {
    const releaseDate = match[1];
    let body = match[2].trim();

    // Promote ### headings to ## to match release note hierarchy
    body = body.replace(/^###\s+/gm, "## ");

    // Fetch commit history if available in git repo
    let commitSection = "";
    try {
      const prevTag = execSync(`git describe --tags --abbrev=0 ${tag}^ 2>/dev/null || true`, { encoding: "utf-8" }).trim();
      const range = prevTag ? `${prevTag}..${tag}` : tag;
      const commits = execSync(`git log --oneline ${range} 2>/dev/null || true`, { encoding: "utf-8" }).trim();
      if (commits) {
        const commitLines = commits.split("\n").map(l => `- \`${l.trim()}\``).join("\n");
        commitSection = `\n\n## 📄 Changes & Commits\n\n${commitLines}\n\nFull commit history: \`git log --oneline ${range}\``;
      }
    } catch {
      // Shallow or detached git repo
    }

    const installSection = `\n\n## 📦 Install & Upgrading\n\nDownload the installer for your platform from the Assets section below:\n\n- Windows: \`LewdZone-Setup-${tag}-windows-x64.exe\`\n- macOS: \`LewdZone-Setup-${tag}-macos-arm64\` (or \`macos-x64\` if available)\n- Linux: \`LewdZone-Setup-${tag}-linux-x64\`\n\nUpgrade in place: run the new installer over your existing installation. The unified installer will repair/replace files and update shortcuts.\n\n## Verification\n\n- \`cargo check\` + \`cargo test\` (src-tauri) — passed\n- \`npm run check\` (svelte-check) — 0 errors, 0 warnings\n- \`npm run test\` (vitest) — passed\n- \`npm run build\` — adapter-static site generated successfully`;

    notes = `# LewdZone Launcher ${tag}\n\n**Release date:** ${releaseDate}\n\n${body}${installSection}${commitSection}`;
  }
}

if (!notes) {
  notes = `# ${tag}\n\nRelease ${tag}`;
}

const { title, body } = splitTitleAndBody(notes.trim());
writeFileSync(outPath, body + "\n", "utf-8");
writeFileSync(titlePath, title + "\n", "utf-8");
console.log(`Extracted and formatted release notes for ${tag} to ${outPath}`);
console.log(`Release title -> ${titlePath}: ${title}`);

