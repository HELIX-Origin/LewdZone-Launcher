<svelte:options runes={true} />

<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { applyTokens } from "$lib/theme/apply";
  import { settingsStore } from "$lib/stores/clientCache";

  type LoadState = "loading" | "ready" | "error";

  let status: LoadState = $state(settingsStore.isLoaded ? "ready" : "loading");
  let error = $state("");

  let snapshot = $state<Record<string, unknown>>(settingsStore.snapshot);
  let themes: string[] = $state(settingsStore.themes);
  let activeTheme = $state(settingsStore.activeTheme);
  let busy = $state(false);
  let toast = $state("");
  let logPath = $state<string | null>(settingsStore.logPath);
  let showLogPreview = $state(false);
  let logPreview = $state("");
  let isRefreshing = $state(false);

  async function load(force = false) {
    if (force) {
      isRefreshing = true;
    } else if (!settingsStore.isLoaded) {
      status = "loading";
    }
    try {
      const [cfg, list, path] = await Promise.all([
        invoke<Record<string, unknown>>("settings_get"),
        invoke<string[]>("themes_list"),
        invoke<string | null>("get_debug_log_path"),
      ]);
      snapshot = cfg;
      themes = list;
      logPath = path;
      const t = cfg["theme"];
      activeTheme = typeof t === "string" && t.trim() ? t : "(default)";
      settingsStore.snapshot = snapshot;
      settingsStore.themes = themes;
      settingsStore.logPath = logPath;
      settingsStore.activeTheme = activeTheme;
      settingsStore.isLoaded = true;
      status = "ready";
    } catch (err) {
      if (!settingsStore.isLoaded) {
        status = "error";
      }
      error = String(err);
    } finally {
      isRefreshing = false;
    }
  }

  onMount(() => {
    load(false);
  });

  async function save(key: string, value: unknown, secret = false) {
    busy = true;
    try {
      const view = await invoke<{ key: string; value: never }>("settings_set", {
        key,
        value: String(value),
        secret,
      });
      snapshot = { ...snapshot, [view.key]: view.value };
      settingsStore.snapshot = snapshot;
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

  let sgdbKeyDraft = $state("");

  async function saveSgdbKey() {
    if (!sgdbKeyDraft.trim()) return;
    await save("sgdb-api-key", sgdbKeyDraft.trim(), true);
    sgdbKeyDraft = "";
    // Refresh snapshot so the status badge updates immediately.
    await load();
  }

  async function clearSgdbKey() {
    await save("sgdb-api-key", "", true);
    // Refresh snapshot so the status badge clears.
    await load();
  }

  let igdbClientIdDraft = $state("");
  let igdbClientSecretDraft = $state("");

  async function saveIgdbClientId() {
    if (!igdbClientIdDraft.trim()) return;
    await save("igdb-client-id", igdbClientIdDraft.trim(), true);
    igdbClientIdDraft = "";
    await load();
  }

  async function clearIgdbClientId() {
    await save("igdb-client-id", "", true);
    await load();
  }

  async function saveIgdbClientSecret() {
    if (!igdbClientSecretDraft.trim()) return;
    await save("igdb-client-secret", igdbClientSecretDraft.trim(), true);
    igdbClientSecretDraft = "";
    await load();
  }

  async function clearIgdbClientSecret() {
    await save("igdb-client-secret", "", true);
    await load();
  }

  async function openLogFolder() {
    try {
      await invoke("open_debug_log_folder");
    } catch (err) {
      toast = `could not open folder: ${err}`;
    }
  }

  async function clearLog() {
    try {
      await invoke("clear_debug_log");
      logPreview = "";
      toast = "Log file cleared";
    } catch (err) {
      toast = `clear failed: ${err}`;
    }
  }

  async function toggleViewLog() {
    if (showLogPreview) {
      showLogPreview = false;
      return;
    }
    try {
      logPreview = await invoke<string>("read_debug_log");
      showLogPreview = true;
    } catch (err) {
      toast = `could not read log: ${err}`;
    }
  }
</script>

{#if status === "loading" && Object.keys(snapshot).length === 0}
  <p class="note">Loading settings…</p>
{:else if status === "error" && Object.keys(snapshot).length === 0}
  <p class="note error">{error}</p>
{:else}
  <div class="settings">
    <div class="settings-head">
      <h1>Settings</h1>
      <button
        type="button"
        class="refresh-btn"
        class:spinning={isRefreshing}
        disabled={isRefreshing}
        onclick={() => load(true)}
        title="Refresh settings"
        aria-label="Refresh settings"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67"/>
        </svg>
        <span>{isRefreshing ? "Refreshing…" : "Refresh"}</span>
      </button>
    </div>

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

    <label class="field">
      <span class="field-label">Extracted games directory</span>
      <input
        class:touched={false}
        type="text"
        placeholder="e.g. D:\Games\LewdZone"
        value={String(snapshot["games-dir"] ?? "")}
        onchange={(e) => save("games-dir", (e.currentTarget as HTMLInputElement).value)}
        onkeydown={(e) => {
          if (e.key === "Enter") (e.currentTarget as HTMLInputElement).blur();
        }}
      />
      <span class="field-hint">Folder where you personally extract games. Use "Scan Games" in the Library to discover installed games here.</span>
    </label>

    <label class="field">
      <span class="field-label">7-Zip console executable path</span>
      <input
        class:touched={false}
        type="text"
        placeholder="e.g. C:\Utilities\7z or C:\Utilities\7z\7za.exe"
        value={String(snapshot["7z-path"] ?? "")}
        onchange={(e) => save("7z-path", (e.currentTarget as HTMLInputElement).value)}
        onkeydown={(e) => {
          if (e.key === "Enter") (e.currentTarget as HTMLInputElement).blur();
        }}
      />
      <span class="field-hint">Path to the 7-Zip command line tool (7za.exe or 7z.exe) for fast multi-format archive extractions.</span>
    </label>

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

    <div class="section-divider">
      <span>Content Providers</span>
    </div>

    <label class="field">
      <span class="field-label">SteamGridDB API key</span>
      <div class="secret-row">
        <input
          type="password"
          class="secret-input"
          placeholder={snapshot["sgdb-api-key"] === "(set)"
            ? "●●●●●●●●●●●●●●●● (saved — paste to replace)"
            : "Paste your SteamGridDB API key here"}
          autocomplete="off"
          oninput={(e) => {
            sgdbKeyDraft = (e.currentTarget as HTMLInputElement).value;
          }}
          onkeydown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();
              saveSgdbKey();
            }
          }}
        />
        <button
          type="button"
          class="secret-save-btn"
          aria-label="Save SteamGridDB API key"
          disabled={!sgdbKeyDraft || busy}
          onclick={saveSgdbKey}
        >
          Save
        </button>
        {#if snapshot["sgdb-api-key"] === "(set)"}
          <button
            type="button"
            class="secret-clear-btn"
            disabled={busy}
            onclick={clearSgdbKey}
            title="Remove saved key"
          >
            Clear
          </button>
        {/if}
      </div>
      <span class="field-hint">
        Used to fetch high-quality cover art, hero banners, and icons for your library.
        Get a free key at <a href="https://www.steamgriddb.com/profile/preferences/api" target="_blank" rel="noreferrer">steamgriddb.com</a>.
        Stored securely in the local SQLite database — never written to any config file.
        {#if snapshot["sgdb-api-key"] === "(set)"}
          <span class="secret-status set">● Key saved</span>
        {:else}
          <span class="secret-status unset">○ Not configured</span>
        {/if}
      </span>
    </label>

    <label class="field">
      <span class="field-label">IGDB (Twitch) Client ID</span>
      <div class="secret-row">
        <input
          type="password"
          class="secret-input"
          placeholder={snapshot["igdb-client-id"] === "(set)"
            ? "●●●●●●●●●●●●●●●● (saved — paste to replace)"
            : "Paste your Twitch Client ID here"}
          autocomplete="off"
          oninput={(e) => {
            igdbClientIdDraft = (e.currentTarget as HTMLInputElement).value;
          }}
          onkeydown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();
              saveIgdbClientId();
            }
          }}
        />
        <button
          type="button"
          class="secret-save-btn"
          aria-label="Save IGDB Client ID"
          disabled={!igdbClientIdDraft || busy}
          onclick={saveIgdbClientId}
        >
          Save
        </button>
        {#if snapshot["igdb-client-id"] === "(set)"}
          <button
            type="button"
            class="secret-clear-btn"
            disabled={busy}
            onclick={clearIgdbClientId}
            title="Remove saved Client ID"
          >
            Clear
          </button>
        {/if}
      </div>
      <span class="field-hint">
        Used alongside Client Secret to enrich game descriptions, genres, screenshots, and artwork via IGDB.
        {#if snapshot["igdb-client-id"] === "(set)"}
          <span class="secret-status set">● Saved</span>
        {:else}
          <span class="secret-status unset">○ Not configured</span>
        {/if}
      </span>
    </label>

    <label class="field">
      <span class="field-label">IGDB (Twitch) Client Secret</span>
      <div class="secret-row">
        <input
          type="password"
          class="secret-input"
          placeholder={snapshot["igdb-client-secret"] === "(set)"
            ? "●●●●●●●●●●●●●●●● (saved — paste to replace)"
            : "Paste your Twitch Client Secret here"}
          autocomplete="off"
          oninput={(e) => {
            igdbClientSecretDraft = (e.currentTarget as HTMLInputElement).value;
          }}
          onkeydown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();
              saveIgdbClientSecret();
            }
          }}
        />
        <button
          type="button"
          class="secret-save-btn"
          aria-label="Save IGDB Client Secret"
          disabled={!igdbClientSecretDraft || busy}
          onclick={saveIgdbClientSecret}
        >
          Save
        </button>
        {#if snapshot["igdb-client-secret"] === "(set)"}
          <button
            type="button"
            class="secret-clear-btn"
            disabled={busy}
            onclick={clearIgdbClientSecret}
            title="Remove saved Client Secret"
          >
            Clear
          </button>
        {/if}
      </div>
      <span class="field-hint">
        Stored securely in the local SQLite database.
        {#if snapshot["igdb-client-secret"] === "(set)"}
          <span class="secret-status set">● Saved</span>
        {:else}
          <span class="secret-status unset">○ Not configured</span>
        {/if}
      </span>
    </label>

    <div class="section-divider">
      <span>Diagnostics & Support</span>
    </div>

    <label class="field checkbox-field">
      <div class="checkbox-row">
        <input
          type="checkbox"
          id="debug-logging-toggle"
          checked={Boolean(snapshot["debug-logging"])}
          onchange={(e) => save("debug-logging", (e.currentTarget as HTMLInputElement).checked)}
        />
        <label for="debug-logging-toggle" class="checkbox-label">
          Enable Debug Logging
        </label>
        {#if snapshot["debug-logging"]}
          <span class="secret-status set">● Active</span>
        {:else}
          <span class="secret-status unset">○ Disabled</span>
        {/if}
      </div>
      <span class="field-hint">
        Log detailed runtime operations and errors to assist with troubleshooting and support requests.
      </span>
    </label>

    {#if logPath}
      <div class="log-card">
        <div class="log-path-row">
          <span class="log-label">Log File:</span>
          <code class="log-path" title={logPath}>{logPath}</code>
        </div>
        <div class="log-actions">
          <button type="button" class="log-btn" onclick={openLogFolder} disabled={busy}>
            Open Log Folder
          </button>
          <button type="button" class="log-btn" onclick={toggleViewLog} disabled={busy}>
            {showLogPreview ? "Hide Log Preview" : "View Recent Logs"}
          </button>
          <button type="button" class="log-btn danger" onclick={clearLog} disabled={busy}>
            Clear Log
          </button>
        </div>
        {#if showLogPreview}
          <div class="log-preview-box">
            <pre class="log-content">{logPreview || "(log file is empty)"}</pre>
          </div>
        {/if}
      </div>
    {/if}

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

  .settings-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  h1 {
    font-size: 20px;
    margin: 0;
  }

  .refresh-btn {
    appearance: none;
    border: 1px solid var(--lz-surface-2);
    background: var(--lz-surface);
    color: var(--lz-text);
    font: inherit;
    font-size: 13px;
    font-weight: 500;
    padding: 6px 12px;
    border-radius: var(--lz-radius);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    transition: all 0.2s ease;
  }

  .refresh-btn:hover:not(:disabled) {
    background: var(--lz-surface-2);
    color: var(--lz-accent);
  }

  .refresh-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .refresh-btn svg {
    width: 14px;
    height: 14px;
    transition: transform 0.2s ease;
  }

  .refresh-btn.spinning svg {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
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

  .section-divider {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--lz-text-dim);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    margin-top: 4px;
  }

  .section-divider::before,
  .section-divider::after {
    content: '';
    flex: 1;
    height: 1px;
    background: var(--lz-border);
  }

  .secret-row {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .secret-input {
    flex: 1;
    background: var(--lz-surface-2);
    border: 1px solid var(--lz-surface-2);
    color: var(--lz-text);
    border-radius: var(--lz-radius);
    padding: 8px 10px;
    font: inherit;
  }

  .secret-input:focus {
    outline: 1px solid var(--lz-accent);
    border-color: var(--lz-accent);
  }

  .secret-save-btn {
    padding: 7px 14px;
    border: none;
    border-radius: var(--lz-radius);
    background: var(--lz-accent);
    color: var(--lz-bg);
    font-weight: 600;
    font-size: 12px;
    cursor: pointer;
    white-space: nowrap;
    transition: opacity 0.15s;
  }

  .secret-save-btn:hover:not(:disabled) {
    opacity: 0.85;
  }

  .secret-save-btn:disabled {
    opacity: 0.45;
    cursor: default;
  }

  .secret-clear-btn {
    padding: 7px 10px;
    border: 1px solid var(--lz-danger);
    border-radius: var(--lz-radius);
    background: transparent;
    color: var(--lz-danger);
    font-weight: 600;
    font-size: 12px;
    cursor: pointer;
    white-space: nowrap;
    transition: all 0.15s;
  }

  .secret-clear-btn:hover:not(:disabled) {
    background: var(--lz-danger);
    color: var(--lz-bg);
  }

  .secret-clear-btn:disabled {
    opacity: 0.45;
    cursor: default;
  }

  .field-hint a {
    color: var(--lz-accent);
    text-decoration: none;
  }

  .field-hint a:hover {
    text-decoration: underline;
  }

  .secret-status {
    display: inline-block;
    margin-left: 6px;
    font-size: 11px;
    font-weight: 600;
  }

  .secret-status.set {
    color: var(--lz-ok);
  }

  .secret-status.unset {
    color: var(--lz-text-dim);
  }

  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .checkbox-label {
    font-weight: 600;
    font-size: 13px;
    cursor: pointer;
    user-select: none;
  }

  .log-card {
    background: var(--lz-surface);
    border: 1px solid var(--lz-surface-2);
    border-radius: var(--lz-radius);
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .log-path-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .log-label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--lz-text-dim);
    font-weight: 600;
  }

  .log-path {
    font-size: 12px;
    background: var(--lz-surface-2);
    padding: 4px 8px;
    border-radius: 4px;
    word-break: break-all;
    user-select: all;
  }

  .log-actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .log-btn {
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 600;
    border-radius: var(--lz-radius);
    border: 1px solid var(--lz-surface-2);
    background: var(--lz-surface-2);
    color: var(--lz-text);
    cursor: pointer;
    transition: all 0.15s;
  }

  .log-btn:hover:not(:disabled) {
    border-color: var(--lz-cyan);
    color: var(--lz-cyan);
  }

  .log-btn.danger {
    color: var(--lz-danger);
    border-color: rgba(255, 92, 92, 0.3);
  }

  .log-btn.danger:hover:not(:disabled) {
    background: var(--lz-danger);
    color: var(--lz-bg);
  }

  .log-preview-box {
    margin-top: 4px;
    max-height: 240px;
    overflow-y: auto;
    background: #0d1117;
    border: 1px solid var(--lz-surface-2);
    border-radius: 4px;
    padding: 8px 12px;
  }

  .log-content {
    margin: 0;
    font-family: monospace;
    font-size: 11px;
    line-height: 1.4;
    color: #c9d1d9;
    white-space: pre-wrap;
    word-break: break-all;
  }
</style>