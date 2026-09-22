import { invoke } from "@tauri-apps/api/core";

/** Apply a token list (Vec<(name, value)>) to :root as CSS custom properties. */
export function applyTokens(tokens: Array<[string, string]>): void {
  const root = document.documentElement;
  for (const [name, value] of tokens) {
    root.style.setProperty(name, value);
  }
}

/** Fetch the effective tokens for the active theme and apply them. */
export async function loadAndApplyTheme(): Promise<void> {
  try {
    const tokens = await invoke<Array<[string, string]>>("themes_tokens");
    applyTokens(tokens);
  } catch {
    // fall back to whatever tokens are already on :root (default.css)
  }
}