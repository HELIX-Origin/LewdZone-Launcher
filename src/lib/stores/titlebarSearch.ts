import { writable } from "svelte/store";

/**
 * Bridges the titlebar search input (in +layout.svelte) with the Store page.
 *
 * `q` is the live query string; `submit` is toggled to true when the user
 * presses Enter so the Store page can trigger a catalog reload.
 */
export const titlebarSearch = writable<{ q: string; submit: boolean }>({
  q: "",
  submit: false,
});
