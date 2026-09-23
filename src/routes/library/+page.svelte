<svelte:options runes={true} />

<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  type LoadState = "loading" | "ready" | "error";

  interface LibraryGame {
    post_id: number;
    size: number;
    folder: string;
  }

  interface LibraryListing {
    root: string | null;
    games: LibraryGame[];
  }

  let status: LoadState = $state("loading");
  let error = $state("");
  let games: LibraryGame[] = $state([]);
  let root: string | null = $state(null);

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

  function folderName(folder: string): string {
    const cleaned = folder.replace(/\\/g, "/");
    const parts = cleaned.split("/").filter(Boolean);
    return parts.length > 0 ? parts[parts.length - 1] : folder;
  }

  onMount(async () => {
    status = "loading";
    try {
      const listing = await invoke<LibraryListing>("library_list");
      games = listing.games;
      root = listing.root;
      status = "ready";
    } catch (err) {
      status = "error";
      error = String(err);
    }
  });
</script>

{#if status === "loading"}
  <p class="note">Reading your library…</p>
{:else if status === "error"}
  <p class="note error">{error}</p>
{:else}
  <section class="library">
    <div class="head">
      <h1>
        <span class="head-icon" aria-hidden="true"
          ><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M5 19V5h4v14H5Z"/><path d="M10 19V5h4v14h-4Z"/><path d="M15 19V5h4v14h-4Z"/></svg></span
        >Library
      </h1>
      {#if root}
        <span class="root" title="Library root">{root}</span>
      {/if}
    </div>

    {#if games.length === 0}
      <p class="note">
        Nothing installed yet. Pick a game in the Store and download it — it will
        appear here ready to play.
      </p>
    {:else}
      <div class="grid" role="list">
        {#each games as game (game.post_id)}
          <div class="tile" role="listitem" title={game.folder}>
            <div
              class="icon"
              aria-hidden="true"
              data-post-id={game.post_id}
            >
              <span class="icon-glyph"
                ><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="6.5" width="18" height="11" rx="5.5"/><circle cx="8" cy="11.5" r="1.1" fill="currentColor" stroke="none"/><circle cx="12.5" cy="11.5" r="1.1" fill="currentColor" stroke="none"/><path d="M16.2 14.4h.01M18.6 12.4h.01"/></svg></span
              >
            </div>
            <div class="tile-body">
              <div class="tile-name">{folderName(game.folder)}</div>
              <div class="tile-meta">
                <span>ID {game.post_id}</span>
                <span>·</span>
                <span>{formatSize(game.size)}</span>
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
    gap: 12px;
    margin-bottom: var(--lz-gap);
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
    cursor: default;
  }

  .icon {
    aspect-ratio: 2 / 3;
    background:
      linear-gradient(135deg, var(--lz-primary), var(--lz-cyan));
    display: flex;
    align-items: center;
    justify-content: center;
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
    margin-top: 4px;
    display: flex;
    gap: 6px;
    color: var(--lz-text-dim);
    font-size: 11px;
  }

  .note {
    color: var(--lz-text-dim);
  }

  .note.error {
    color: var(--lz-danger);
  }
</style>