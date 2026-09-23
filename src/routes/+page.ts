import { redirect } from "@sveltejs/kit";
import { invoke } from "@tauri-apps/api/core";

const VALID_HOMES = ["store", "favorites", "library", "downloads", "settings"];

export async function load() {
  let home = "store";
  try {
    const value = await invoke<string | null>("settings_get", { key: "home-page" });
    if (value && VALID_HOMES.includes(value)) home = value;
  } catch {
    // default to the store when the setting cannot be read (e.g. tests)
  }
  redirect(307, `/${home}`);
}