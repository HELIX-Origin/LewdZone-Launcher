/**
 * Formatting utilities for game titles, archive names, and release descriptors.
 * Mirrors the canonical formatting logic in `src-tauri/src/core/library.rs`.
 */

export function toTitleCase(s: string): string {
  if (!s) return "";
  return s
    .split(/[-_\s]+/)
    .filter((part) => part.length > 0)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}

export function cleanDisplayTitle(raw: string): string {
  if (!raw) return "";
  let clean = raw;
  while (clean.includes("[") && clean.includes("]")) {
    clean = clean.replace(/\[[^\]]*\]/g, " ");
  }
  while (clean.includes("(") && clean.includes(")")) {
    clean = clean.replace(/\([^)]*\)/g, " ");
  }
  clean = clean.replace(/_/g, " ");

  const lower = clean.toLowerCase();
  for (const marker of [
    " - version",
    " version ",
    " - v",
    " - chapter",
    " chapter ",
    " - ch.",
    " ch. ",
    " - ch ",
    " - episode",
    " episode ",
    " - ep.",
    " ep. ",
    " - ep ",
    " - part",
    " part ",
    " - season",
    " season ",
    " - build",
    " build ",
  ]) {
    const idx = lower.indexOf(marker);
    if (idx !== -1) {
      clean = clean.slice(0, idx);
      break;
    }
  }

  // Trailing platform markers
  for (const suffix of [
    " - pc",
    " - mac",
    " - linux",
    " pc",
    " mac",
    " linux",
    " - windows",
  ]) {
    if (clean.toLowerCase().endsWith(suffix)) {
      clean = clean.slice(0, clean.length - suffix.length);
      break;
    }
  }

  let res = clean.replace(/\s+/g, " ").trim();
  if (!res) res = raw.trim();

  // If result looks like a raw kebab-case slug (e.g. "dating-my-daughter"),
  // convert it to Title Case words (e.g. "Dating My Daughter")
  if (
    /^[a-z0-9-]+$/.test(res) &&
    res.includes("-") &&
    !res.includes(" ")
  ) {
    res = toTitleCase(res);
  }

  return res;
}

export function formatReleaseDescriptor(rawTitle: string, versionInput: string): string {
  const combined = `${(versionInput || "").trim()} ${(rawTitle || "").trim()}`.trim();
  const lower = combined.toLowerCase();

  // 1. Detect chapter / episode / part / season / build
  let chapterInfo: { wording: string; count: string } | null = null;
  const prefixes: [string, string][] = [
    ["chapter", "Chapter"],
    ["ch.", "Chapter"],
    ["ch ", "Chapter"],
    ["episode", "Episode"],
    ["ep.", "Episode"],
    ["ep ", "Episode"],
    ["part", "Part"],
    ["season", "Season"],
    ["build", "Build"],
  ];

  for (const [prefix, wording] of prefixes) {
    const idx = lower.indexOf(prefix);
    if (idx !== -1) {
      const after = combined.slice(idx + prefix.length).trimStart();
      const numMatch = after.match(/^[\d.-]+/);
      if (numMatch) {
        const cleanNum = numMatch[0].replace(/^[.-]+|[.-]+$/g, "");
        if (cleanNum) {
          chapterInfo = { wording, count: cleanNum };
          break;
        }
      }
    }
  }

  // 2. Detect version number
  let versionNumber: string | null = null;

  for (const token of combined.split(/\s+/)) {
    const tClean = token.replace(/^[^a-zA-Z0-9.]+|[^a-zA-Z0-9.]+$/g, "");
    const tLower = tClean.toLowerCase();
    if (tLower.startsWith("v") && tLower.length > 1 && !tLower.startsWith("ver")) {
      const candidate = tClean.slice(1).replace(/^\.+/, "");
      if (/^\d/.test(candidate)) {
        const vMatch = candidate.match(/^[\d.-]+/);
        if (vMatch) {
          const vClean = vMatch[0].replace(/^[.-]+|[.-]+$/g, "");
          if (vClean) {
            versionNumber = vClean;
            break;
          }
        }
      }
    }
  }

  if (!versionNumber) {
    const vIdx = lower.indexOf("version");
    if (vIdx !== -1) {
      let after = combined.slice(vIdx + "version".length).trimStart();
      after = after.replace(/^[:.\s]+/, "").replace(/^[vV]/, "");
      const vMatch = after.match(/^[\d.-]+/);
      if (vMatch) {
        const vClean = vMatch[0].replace(/^[.-]+|[.-]+$/g, "");
        if (vClean && vClean !== "latest") {
          versionNumber = vClean;
        }
      }
    }
  }

  if (!versionNumber) {
    const vInputClean = (versionInput || "").trim().replace(/^[vV]/, "");
    if (/^\d/.test(vInputClean) && (vInputClean.includes(".") || !chapterInfo)) {
      const vMatch = vInputClean.match(/^[\d.-]+/);
      if (vMatch) {
        const vClean = vMatch[0].replace(/^[.-]+|[.-]+$/g, "");
        if (vClean && vClean !== "latest") {
          versionNumber = vClean;
        }
      }
    }
  }

  if (versionNumber && chapterInfo && versionNumber === chapterInfo.count) {
    const vLower = (versionInput || "").toLowerCase();
    if (!vLower.includes("v") && !vLower.includes("version")) {
      versionNumber = null;
    }
  }

  if (versionNumber && chapterInfo) {
    return `Version ${versionNumber} ${chapterInfo.wording} ${chapterInfo.count}`;
  } else if (versionNumber) {
    return `Version ${versionNumber}`;
  } else if (chapterInfo) {
    return `${chapterInfo.wording} ${chapterInfo.count}`;
  } else {
    return "Version 1.0";
  }
}

export function formatCanonicalTitle(
  rawTitle: string,
  statusHint?: string,
  versionInput?: string
): string {
  const cleanTitle = cleanDisplayTitle(rawTitle);
  const title = cleanTitle || (rawTitle || "").trim();

  let status = "Unknown";
  if (statusHint && statusHint.trim()) {
    const h = statusHint.toLowerCase();
    if (h.includes("finish")) status = "Finished";
    else if (h.includes("ongoing")) status = "Ongoing";
    else if (h.includes("abandon")) status = "Abandoned";
    else if (h.includes("complet")) status = "Completed";
    else if (h.includes("on hold") || h.includes("onhold")) status = "On Hold";
    else if (h.includes("hiatus")) status = "Hiatus";
    else if (h === "unknown") status = "Unknown";
    else status = statusHint.charAt(0).toUpperCase() + statusHint.slice(1);
  } else {
    const bracketMatch = (rawTitle || "").match(/\[([^\]]+)\]/);
    if (bracketMatch) {
      const inside = bracketMatch[1].trim();
      const lower = inside.toLowerCase();
      if (lower.includes("ongoing")) status = "Ongoing";
      else if (lower.includes("finish")) status = "Finished";
      else if (lower.includes("abandon")) status = "Abandoned";
      else if (lower.includes("complet")) status = "Completed";
      else if (lower.includes("on hold") || lower.includes("onhold")) status = "On Hold";
      else if (lower.includes("hiatus")) status = "Hiatus";
      else if (lower === "unknown") status = "Unknown";
      else if (inside) status = inside.charAt(0).toUpperCase() + inside.slice(1);
    }
  }

  const releaseDesc = formatReleaseDescriptor(rawTitle, versionInput || "");
  return `${title} [${status}] - ${releaseDesc}`;
}

export function formatDisplayMessage(msg: string): string {
  if (!msg) return "";
  if (
    msg.includes("[") ||
    /version/i.test(msg) ||
    /chapter/i.test(msg) ||
    /ch\./i.test(msg)
  ) {
    return formatCanonicalTitle(msg);
  }
  return msg;
}

