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

  const SECRET_KEYS = [
    ["sgdb-api-key", "SteamGridDB API key", "Used to fetch artwork (https://www.steamgriddb.com/profile/preferences/api)."],
    ["igdb-client-id", "IGDB client ID", "Twitch/IGDB OAuth client ID."],
    ["igdb-client-secret", "IGDB client secret", "Twitch/IGDB OAuth client secret."],
  ] as const;

  function secretPresence(key: string) {
    const v = snapshot[key];
    return typeof v === "string" && v === "(set)" ? "(set)" : "(not set)";
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

{#each [
      ["library-root", "Library root", "Where games are installed (library folders)."],
      ["content-priority", "Content providers", "Comma-separated provider priority list."],
      [
        "source-priority",
        "Preferred download sources",
        "Comma-separated cloud hosts (mega, google, dropbox, mediafire, pixeldrain). Preferred sources are listed first and used as the default on a game's download panel.",
      ],
    ] as [key, label, hint] (key)}
      <label class="field">
        <span class="field-label">{label}</span>
        <input
          class:touched={false}
          type="text"
          value={String(snapshot[key] ?? "")}
          onchange={(e) => save(key, (e.currentTarget as HTMLInputElement).value)}
          onkeydown={(e) => {
            if (e.key === "Enter") (e.currentTarget as HTMLInputElement).blur();
          }}
        />
        <span class="field-hint">{hint}</span>
      </label>
    {/each}

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

    <fieldset class="field">
      <legend class="field-label">API keys</legend>
      <p class="field-hint">
        Stored in the local database — never in the readable config file. Leave blank and press Enter to clear a key.
      </p>
      {#each SECRET_KEYS as [key, label, hint] (key)}
        <label class="field">
          <span class="field-label">{label}</span>
          <input
            type="password"
            value=""
            placeholder={secretPresence(key)}
            aria-label={label}
            onchange={(e) => save(key, (e.currentTarget as HTMLInputElement).value, true)}
            onkeydown={(e) => {
              if (e.key === "Enter") (e.currentTarget as HTMLInputElement).blur();
            }}
          />
          <span class="field-hint">{hint}</span>
        </label>
      {/each}
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
  }

  .field-label {
    font-weight: 600;
    font-size: 13px;
    color: var(--lz-text-dim);
  }

  input[type="text"],
  input[type="password"],
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