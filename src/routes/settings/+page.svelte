<svelte:options runes={true} />

<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { applyTokens } from "$lib/theme/apply";

  type LoadState = "loading" | "ready" | "error";

  let status: LoadState = $state("loading");
  let error = $state("");

  let snapshot = $state<Record<string, unknown>>({});
  let themes: string[] = $state([]);
  let activeTheme = $state("(default)");
  let busy = $state(false);
  let toast = $state("");

  const AVAILABLE_SOURCES = [
    { id: "mega", label: "MEGA" },
    { id: "google", label: "Google Drive" },
    { id: "dropbox", label: "Dropbox" },
    { id: "mediafire", label: "MediaFire" },
    { id: "pixeldrain", label: "PixelDrain" },
    { id: "workupload", label: "Workupload" },
    { id: "fileknot", label: "Fileknot" },
    { id: "transfaze", label: "Transfaze" },
    { id: "uploadhaven", label: "UploadHaven" },
    { id: "mixdrop", label: "MixDrop" },
    { id: "racaty", label: "Racaty" },
    { id: "terminal", label: "Terminal" },
  ] as const;

  function parsePreferredSources(raw: unknown): string[] {
    if (typeof raw !== "string" || !raw.trim()) return [];
    return raw
      .split(",")
      .map((s) => s.trim().toLowerCase())
      .filter((s) => s.length > 0);
  }

  function toggleSource(sourceId: string) {
    const current = parsePreferredSources(snapshot["source-priority"]);
    let updated: string[];
    if (current.includes(sourceId)) {
      updated = current.filter((id) => id !== sourceId);
    } else {
      updated = [...current, sourceId];
    }
    save("source-priority", updated.join(","));
  }

  async function load() {
    status = "loading";
    try {
      const [cfg, list] = await Promise.all([
        invoke<Record<string, unknown>>("settings_get"),
        invoke<string[]>("themes_list"),
      ]);
      snapshot = cfg;
      themes = list;
      const t = cfg["theme"];
      activeTheme = typeof t === "string" && t.trim() ? t : "(default)";
      status = "ready";
    } catch (err) {
      status = "error";
      error = String(err);
    }
  }

  onMount(load);

  async function save(key: string, value: unknown, secret = false) {
    busy = true;
    try {
      const view = await invoke<{ key: string; value: never }>("settings_set", {
        key,
        value: String(value),
        secret,
      });
      snapshot = { ...snapshot, [view.key]: view.value };
      toast = `${view.key} saved`;
    } catch (err) {
      toast = `failed: ${err}`;
    } finally {
      busy = false;
    }
  }

  async function applyTheme(name: string) {
    busy = true;
    try {
      const tokens = await invoke<Array<[string, string]>>("themes_apply", {
        name,
      });
      applyTokens(tokens);
      activeTheme = name === "" ? "(default)" : name;
      toast = `theme applied${name ? `: ${name}` : ""}`;
    } catch (err) {
      toast = `theme failed: ${err}`;
    } finally {
      busy = false;
    }
  }
</script>

{#if status === "loading"}
  <p class="note">Loading settings…</p>
{:else if status === "error"}
  <p class="note error">{error}</p>
{:else}
  <div class="settings">
    <h1>Settings</h1>

    <label class="field">
      <span class="field-label">Library root</span>
      <input
        class:touched={false}
        type="text"
        value={String(snapshot["library-root"] ?? "")}
        onchange={(e) => save("library-root", (e.currentTarget as HTMLInputElement).value)}
        onkeydown={(e) => {
          if (e.key === "Enter") (e.currentTarget as HTMLInputElement).blur();
        }}
      />
      <span class="field-hint">Where games are installed (library folders).</span>
    </label>

    <fieldset class="field">
      <legend class="field-label">Preferred download sources</legend>
      <div class="source-toggles" role="group" aria-label="Preferred download sources">
        {#each AVAILABLE_SOURCES as src (src.id)}
          {@const active = parsePreferredSources(snapshot["source-priority"]).includes(src.id)}
          <button
            type="button"
            class="source-toggle-btn"
            class:active={active}
            aria-pressed={active}
            aria-label={`Toggle ${src.label} preference`}
            onclick={() => toggleSource(src.id)}
          >
            <span class="toggle-indicator">{active ? "✓" : "+"}</span>
            <span class="source-name">{src.label}</span>
          </button>
        {/each}
      </div>
      <span class="field-hint">
        Toggle cloud hosts to mark as preferred. Preferred sources appear first and serve as default on a game's download panel.
      </span>
    </fieldset>

    <label class="field">
      <span class="field-label">Home page</span>
      <select
        aria-label="Home page"
        value={typeof snapshot["home-page"] === "string" && snapshot["home-page"]
          ? snapshot["home-page"]
          : "store"}
        onchange={(e) => save("home-page", (e.currentTarget as HTMLSelectElement).value)}
      >
        {#each ["store", "favorites", "library", "downloads", "settings"] as page (page)}
          <option value={page}>{page}</option>
        {/each}
      </select>
      <span class="field-hint">Which tab opens at launch.</span>
    </label>

    <label class="field">
      <span class="field-label">Capture-aware sync</span>
      <input
        type="checkbox"
        checked={snapshot["capture-aware"] === true}
        onchange={(e) => save("capture-aware", (e.currentTarget as HTMLInputElement).checked)}
      />
      <span class="field-hint">Pause network work while the window is captured/streaming.</span>
    </label>

    <fieldset class="field">
      <legend class="field-label">Theme</legend>
      <select
        value={activeTheme}
        onchange={(e) => applyTheme((e.currentTarget as HTMLSelectElement).value)}
      >
        <option value="">(default)</option>
        {#each themes as name (name)}
          <option value={name}>{name}</option>
        {/each}
      </select>
      <span class="field-hint">Nord, Dracula, and Material ship with the app. Custom skins go in the user skins folder (one subfolder per theme) — next to the app on Windows, in the app data folder on macOS/Linux. Built-in default stays; skins apply at runtime — no restart needed.</span>
    </fieldset>

    {#if toast}<p class="toast" aria-live="polite">{toast}</p>{/if}
  </div>
{/if}

<style>
  .settings {
    max-width: 640px;
    padding: var(--lz-gap);
    display: flex;
    flex-direction: column;
    gap: var(--lz-gap);
  }

  h1 {
    font-size: 20px;
    margin: 0;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    border: none;
    padding: 0;
    margin: 0;
  }

  .field-label {
    font-weight: 600;
    font-size: 13px;
    color: var(--lz-text-dim);
  }

  input[type="text"],
  select {
    background: var(--lz-surface-2);
    border: 1px solid var(--lz-surface-2);
    color: var(--lz-text);
    border-radius: var(--lz-radius);
    padding: 8px 10px;
    font: inherit;
  }

  input[type="text"]:focus,
  select:focus {
    outline: 1px solid var(--lz-accent);
    border-color: var(--lz-accent);
  }

  .source-toggles {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 4px;
  }

  .source-toggle-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: var(--lz-radius);
    border: 1px solid var(--lz-surface-2);
    background: var(--lz-surface-2);
    color: var(--lz-text-dim);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .source-toggle-btn:hover {
    color: var(--lz-text);
    border-color: var(--lz-accent);
  }

  .source-toggle-btn.active {
    background: var(--lz-accent);
    color: #fff;
    border-color: var(--lz-accent);
    font-weight: 600;
  }

  .toggle-indicator {
    font-size: 11px;
    font-weight: bold;
  }

  .field-hint {
    font-size: 12px;
    color: var(--lz-text-dim);
  }

  input[type="checkbox"] {
    accent-color: var(--lz-accent);
    width: 16px;
    height: 16px;
  }

  .toast {
    color: var(--lz-ok);
    margin: 0;
    font-size: 13px;
  }

  .note {
    padding: var(--lz-gap);
    color: var(--lz-text-dim);
  }

  .note.error {
    color: var(--lz-danger);
  }
</style>