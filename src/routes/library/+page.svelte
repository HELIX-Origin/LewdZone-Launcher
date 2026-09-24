<svelte:options runes={true} />

<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  type LoadState = "loading" | "ready" | "error";

  interface LibraryGame {
    slug: string;
    post_id: number | null;
    title: string;
    version: string;
    platform: string;
    tab: string;
    engine: string | null;
    install_path: string;
    candidates: string[];
    launch_exe: string;
    installed_at: string | null;
    size_on_disk: number;
    playtime_seconds?: number;
    play_count?: number;
    last_played_at?: string | null;
  }

  interface LibraryListing {
    root: string | null;
    games: LibraryGame[];
  }

  interface FavoriteCard {
    slug: string;
  }

  let status: LoadState = $state("loading");
  let error = $state("");
  let games: LibraryGame[] = $state([]);
  let root: string | null = $state(null);
  let launching = $state<Record<string, boolean>>({});
  let coverUrls: Record<string, string | null> = $state({});
  let favorites: Record<string, boolean> = $state({});
  let togglingFavorite = $state<Record<string, boolean>>({});
  let sortBy = $state<"alpha" | "recent" | "playtime" | "installed">("alpha");

  let sortedGames = $derived.by(() => {
    const list = [...games];
    if (sortBy === "alpha") {
      return list.sort((a, b) => a.title.localeCompare(b.title));
    } else if (sortBy === "recent") {
      return list.sort((a, b) => {
        const timeA = a.last_played_at ? new Date(a.last_played_at).getTime() : 0;
        const timeB = b.last_played_at ? new Date(b.last_played_at).getTime() : 0;
        return timeB - timeA;
      });
    } else if (sortBy === "playtime") {
      return list.sort((a, b) => (b.playtime_seconds ?? 0) - (a.playtime_seconds ?? 0));
    } else if (sortBy === "installed") {
      return list.sort((a, b) => {
        const timeA = a.installed_at ? new Date(a.installed_at).getTime() : 0;
        const timeB = b.installed_at ? new Date(b.installed_at).getTime() : 0;
        return timeB - timeA;
      });
    }
    return list;
  });

  function formatPlaytime(seconds: number | undefined): string {
    if (!seconds || seconds <= 0) return "Unplayed";
    if (seconds < 60) return `${seconds}s`;
    const mins = Math.floor(seconds / 60);
    if (mins < 60) return `${mins}m`;
    const hours = (seconds / 3600).toFixed(1);
    return `${hours}h`;
  }

  function formatLastPlayed(iso: string | null | undefined): string {
    if (!iso) return "Never";
    try {
      const date = new Date(iso);
      const now = new Date();
      const diffMs = now.getTime() - date.getTime();
      const diffSecs = Math.floor(diffMs / 1000);
      if (diffSecs < 60) return "Just now";
      const diffMins = Math.floor(diffSecs / 60);
      if (diffMins < 60) return `${diffMins}m ago`;
      const diffHours = Math.floor(diffMins / 60);
      if (diffHours < 24) return `${diffHours}h ago`;
      const diffDays = Math.floor(diffHours / 24);
      if (diffDays === 1) return "Yesterday";
      if (diffDays < 7) return `${diffDays}d ago`;
      return date.toLocaleDateString();
    } catch {
      return iso;
    }
  }

  function formatSize(bytes: number): string {
    if (bytes <= 0) return "—";
    const units = ["B", "KB", "MB", "GB", "TB"];
    let n = bytes;
    let i = 0;
    while (n >= 1024 && i < units.length - 1) {
      n /= 1024;
      i += 1;
    }
    return `${n.toFixed(n >= 100 ? 0 : 1)} ${units[i]}`;
  }

  function platformLabel(p: string): string {
    const map: Record<string, string> = {
      pc: "PC",
      mac: "Mac",
      linux: "Linux",
    };
    return map[p.toLowerCase()] ?? p;
  }

  async function load() {
    status = "loading";
    error = "";
    try {
      const listing = await invoke<LibraryListing>("library_list");
      games = listing.games;
      root = listing.root;
      await Promise.all([loadArtwork(listing.games), loadFavorites()]);
      status = "ready";
    } catch (err) {
      status = "error";
      error = String(err);
    }
  }

  async function loadFavorites() {
    try {
      const rows = await invoke<FavoriteCard[]>("favorites_list");
      const next: Record<string, boolean> = {};
      for (const row of rows) {
        next[row.slug] = true;
      }
      favorites = next;
    } catch {
      favorites = {};
    }
  }

  async function loadArtwork(list: LibraryGame[]) {
    const next: Record<string, string | null> = {};
    await Promise.all(
      list.map(async (game) => {
        try {
          const url = await invoke<string | null>("artwork_url", {
            card: {
              slug: game.slug,
              post_id: game.post_id,
              title: game.title,
            },
            kind: "cover",
          });
          next[game.slug] = url ?? null;
        } catch {
          next[game.slug] = null;
        }
      }),
    );
    coverUrls = next;
  }

  async function launch(game: LibraryGame) {
    launching[game.slug] = true;
    try {
      await invoke("game_launch", { slug: game.slug });
    } catch (err) {
      error = String(err);
    } finally {
      launching[game.slug] = false;
    }
  }

  let creatingShortcut = $state<Record<string, boolean>>({});
  let shortcutToast = $state<string | null>(null);

  async function makeShortcut(game: LibraryGame) {
    creatingShortcut[game.slug] = true;
    try {
      await invoke<string>("create_shortcut", { slug: game.slug });
      shortcutToast = `Created shortcut for ${game.title}`;
      setTimeout(() => {
        if (shortcutToast?.includes(game.title)) shortcutToast = null;
      }, 4000);
    } catch (err) {
      error = `Failed to create shortcut: ${err}`;
    } finally {
      creatingShortcut[game.slug] = false;
    }
  }

  async function toggleFavorite(game: LibraryGame) {
    if (togglingFavorite[game.slug]) return;
    togglingFavorite[game.slug] = true;
    try {
      if (favorites[game.slug]) {
        await invoke("favorite_remove", { slug: game.slug });
        favorites[game.slug] = false;
      } else {
        await invoke("favorite_add", { slug: game.slug });
        favorites[game.slug] = true;
      }
      window.dispatchEvent(new CustomEvent("favorites-changed"));
    } catch (err) {
      error = String(err);
    } finally {
      togglingFavorite[game.slug] = false;
    }
  }

  let scanning = $state(false);
  let scanReport = $state<string | null>(null);

  interface ScanReportResult {
    scanned_dir: string;
    found_count: number;
    queued_count: number;
    added_games: string[];
  }

  async function scanGames() {
    scanning = true;
    scanReport = null;
    try {
      const res = await invoke<ScanReportResult>("library_scan");
      if (res.queued_count > 0) {
        scanReport = `Scan complete: ${res.found_count} game(s) found, ${res.queued_count} archive extraction(s) queued. View progress in Queue.`;
      } else if (res.found_count > 0) {
        scanReport = `Scan complete: ${res.found_count} game(s) registered from ${res.scanned_dir}`;
      } else {
        scanReport = `Scan complete: no new games found in ${res.scanned_dir}`;
      }
      await load();
    } catch (err) {
      scanReport = `Scan failed: ${err}`;
    } finally {
      scanning = false;
    }
  }

  onMount(() => {
    load();
    const handler = () => loadFavorites();
    window.addEventListener("favorites-changed", handler);
    return () => window.removeEventListener("favorites-changed", handler);
  });
</script>

{#if status === "loading"}
  <p class="note">Reading your library…</p>
{:else if status === "error"}
  <p class="note error">{error}</p>
{:else}
  <section class="library">
    <div class="head">
      <div class="head-left">
        <h1>
          <span class="head-icon" aria-hidden="true"
            ><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M5 19V5h4v14H5Z"/><path d="M10 19V5h4v14h-4Z"/><path d="M15 19V5h4v14h-4Z"/></svg></span
          >Library
        </h1>
        {#if root}
          <span class="root" title="Library root">{root}</span>
        {/if}
      </div>
      <div class="head-actions">
        <div class="sort-box">
          <label for="sort-select" class="sort-label">Sort:</label>
          <select id="sort-select" class="sort-select" bind:value={sortBy}>
            <option value="alpha">A – Z</option>
            <option value="recent">Recently Played</option>
            <option value="playtime">Most Played</option>
            <option value="installed">Recently Installed</option>
          </select>
        </div>
        <button
          type="button"
          class="scan-btn"
          disabled={scanning}
          onclick={scanGames}
          title="Scan your custom games directory for archives and extracted games"
        >
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/><line x1="11" y1="8" x2="11" y2="14"/><line x1="8" y1="11" x2="14" y2="11"/></svg>
          {scanning ? "Scanning…" : "Scan Games"}
        </button>
      </div>
    </div>

    {#if scanReport}
      <div class="scan-toast" role="status">
        <span>{scanReport}</span>
        <button type="button" class="toast-close" onclick={() => (scanReport = null)} aria-label="Dismiss">×</button>
      </div>
    {/if}

    {#if shortcutToast}
      <div class="shortcut-toast" role="status">
        <span>{shortcutToast}</span>
        <button type="button" class="toast-close" onclick={() => (shortcutToast = null)} aria-label="Dismiss">×</button>
      </div>
    {/if}

    {#if games.length === 0}
      <p class="note">
        Nothing installed yet. Pick a game in the Store and download it — it will
        appear here ready to play.
      </p>
    {:else}
      <div class="grid" role="list">
        {#each sortedGames as game (game.slug)}
          <div class="tile" role="listitem" title={game.install_path}>
            <div class="tile-cover">
              <div
                class="icon"
                aria-hidden="true"
                data-post-id={game.post_id ?? ""}
              >
                {#if coverUrls[game.slug]}
                  <img src={coverUrls[game.slug]} alt="" loading="lazy" />
                {:else}
                  <span class="icon-glyph"
                    ><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="6.5" width="18" height="11" rx="5.5"/><circle cx="8" cy="11.5" r="1.1" fill="currentColor" stroke="none"/><circle cx="12.5" cy="11.5" r="1.1" fill="currentColor" stroke="none"/><path d="M16.2 14.4h.01M18.6 12.4h.01"/></svg></span
                  >
                {/if}
              </div>
              <button
                class="heart-btn"
                class:filled={favorites[game.slug]}
                onclick={() => toggleFavorite(game)}
                disabled={togglingFavorite[game.slug]}
                aria-label={favorites[game.slug]
                  ? `Remove ${game.title} from favorites`
                  : `Add ${game.title} to favorites`}
                title={favorites[game.slug] ? "Remove favorite" : "Add favorite"}
              >
                <svg viewBox="0 0 24 24" fill={favorites[game.slug] ? "currentColor" : "none"} stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M12 20.5 4.6 13a4.8 4.8 0 0 1 0-6.9 5.1 5.1 0 0 1 7.4 0l.6.6.6-.6a5.1 5.1 0 0 1 7.4 0 4.8 4.8 0 0 1 0 6.9L12 20.5Z"/>
                </svg>
              </button>
            </div>
            <div class="tile-body">
              <div class="tile-name">{game.title}</div>
              <div class="tile-meta">
                <span>{game.version}</span>
                <span>·</span>
                <span>{platformLabel(game.platform)}</span>
                <span>·</span>
                <span>{formatSize(game.size_on_disk)}</span>
              </div>
              <div class="tile-stats">
                <span class="badge-playtime">{formatPlaytime(game.playtime_seconds)}</span>
                {#if game.last_played_at}
                  <span class="meta-dot">·</span>
                  <span class="last-played" title={game.last_played_at}>Played {formatLastPlayed(game.last_played_at)}</span>
                {/if}
              </div>
              <div class="btn-row">
                <button
                  class="launch-btn"
                  onclick={() => launch(game)}
                  disabled={launching[game.slug]}
                  aria-label={`Launch ${game.title}`}
                >
                  {launching[game.slug] ? "Launching…" : "Launch"}
                </button>
                <button
                  class="shortcut-btn"
                  onclick={() => makeShortcut(game)}
                  disabled={creatingShortcut[game.slug]}
                  title="Create Desktop / Start menu shortcut"
                  aria-label={`Create shortcut for ${game.title}`}
                >
                  <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                    <polyline points="15 3 21 3 21 9"></polyline>
                    <line x1="10" y1="14" x2="21" y2="3"></line>
                  </svg>
                </button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </section>
{/if}

<style>
  .library {
    padding: var(--lz-gap);
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: var(--lz-gap);
    flex-wrap: wrap;
  }

  .head-left {
    display: flex;
    align-items: baseline;
    gap: 12px;
    flex-wrap: wrap;
    min-width: 0;
  }

  .head-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .sort-box {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: var(--lz-surface-2);
    border: 1px solid var(--lz-border);
    border-radius: var(--lz-radius);
    padding: 3px 8px;
  }

  .sort-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--lz-text-dim);
  }

  .sort-select {
    background: transparent;
    border: none;
    color: var(--lz-text);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    outline: none;
  }

  .sort-select option {
    background: var(--lz-bg);
    color: var(--lz-text);
  }

  .scan-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 14px;
    font-size: 13px;
    font-weight: 600;
    color: var(--lz-text);
    background: var(--lz-surface-2);
    border: 1px solid var(--lz-cyan);
    border-radius: var(--lz-radius);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .scan-btn:hover:not(:disabled) {
    background: var(--lz-cyan);
    color: var(--lz-bg);
  }

  .scan-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .scan-toast {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 14px;
    margin-bottom: var(--lz-gap);
    background: var(--lz-surface-2);
    border: 1px solid var(--lz-cyan);
    border-radius: var(--lz-radius);
    font-size: 13px;
    color: var(--lz-text);
  }

  .toast-close {
    background: transparent;
    border: none;
    color: var(--lz-text-dim);
    font-size: 18px;
    cursor: pointer;
    line-height: 1;
    padding: 0 4px;
  }

  .toast-close:hover {
    color: var(--lz-text);
  }

  .head h1 {
    font-size: 18px;
    margin: 0;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .head-icon :global(svg) {
    width: 22px;
    height: 22px;
    color: var(--lz-cyan);
    display: block;
  }

  .root {
    color: var(--lz-text-dim);
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: var(--lz-gap);
  }

  .tile {
    border-radius: var(--lz-radius);
    background: var(--lz-surface);
    border: 1px solid var(--lz-surface-2);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .tile-cover {
    position: relative;
  }

  .icon {
    aspect-ratio: 2 / 3;
    background:
      linear-gradient(135deg, var(--lz-primary), var(--lz-cyan));
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  .heart-btn {
    position: absolute;
    top: 6px;
    right: 6px;
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.45);
    color: var(--lz-bg);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0;
    transition: transform 0.1s ease, background 0.15s ease;
  }

  .heart-btn:hover:not(:disabled) {
    background: rgba(0, 0, 0, 0.65);
    transform: scale(1.08);
  }

  .heart-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .heart-btn.filled {
    color: var(--lz-danger);
  }

  .heart-btn :global(svg) {
    width: 16px;
    height: 16px;
  }

  .icon img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .icon-glyph {
    color: var(--lz-bg);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .icon-glyph :global(svg) {
    width: 44px;
    height: 44px;
  }

  .tile-body {
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .tile-name {
    font-weight: 600;
    font-size: 12px;
    line-height: 1.2;
    line-clamp: 2;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .tile-meta {
    display: flex;
    gap: 6px;
    color: var(--lz-text-dim);
    font-size: 11px;
    flex-wrap: wrap;
  }

  .tile-stats {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 10.5px;
    color: var(--lz-text-dim);
    flex-wrap: wrap;
  }

  .badge-playtime {
    display: inline-block;
    padding: 1px 6px;
    border-radius: 4px;
    background: var(--lz-surface-2);
    color: var(--lz-cyan);
    font-weight: 600;
  }

  .meta-dot {
    opacity: 0.5;
  }

  .last-played {
    color: var(--lz-text-dim);
  }

  .btn-row {
    display: flex;
    gap: 6px;
    margin-top: 4px;
  }

  .launch-btn {
    flex: 1;
    padding: 6px 10px;
    border: none;
    border-radius: var(--lz-radius);
    background: var(--lz-primary);
    color: var(--lz-bg);
    font-weight: 600;
    font-size: 12px;
    cursor: pointer;
    transition: opacity 0.15s ease;
  }

  .launch-btn:hover:not(:disabled) {
    opacity: 0.85;
  }

  .launch-btn:disabled {
    opacity: 0.6;
    cursor: default;
  }

  .shortcut-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 6px 9px;
    border: 1px solid var(--lz-border);
    border-radius: var(--lz-radius);
    background: var(--lz-surface-2);
    color: var(--lz-text-dim);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .shortcut-btn:hover:not(:disabled) {
    color: var(--lz-cyan);
    border-color: var(--lz-cyan);
    background: var(--lz-surface);
  }

  .shortcut-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .shortcut-toast {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 14px;
    margin-bottom: var(--lz-gap);
    background: var(--lz-surface-2);
    border: 1px solid var(--lz-cyan);
    border-radius: var(--lz-radius);
    font-size: 13px;
    color: var(--lz-text);
  }

  .note {
    color: var(--lz-text-dim);
  }

  .note.error {
    color: var(--lz-danger);
  }
</style>
