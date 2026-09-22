<svelte:options runes={true} />

<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  type LoadState = "loading" | "ready" | "error";

  interface GameCard {
    slug: string;
    title: string;
    thumb_url: string;
    platforms: string[];
    engine: string;
    state: string;
    version_tag: string;
    developer: string;
    genres: string[];
    views: string;
  }

  interface ArchiveMeta {
    page: number;
    total_pages: number | null;
  }

  interface ArchivePage {
    games: GameCard[];
    meta: ArchiveMeta;
  }

  let status: LoadState = $state("loading");
  let error = $state("");
  let page = $state(1);
  let totalPages: number | null = $state(null);
  let games: GameCard[] = $state([]);

  async function load(p: number) {
    status = "loading";
    try {
      const archive = await invoke<ArchivePage>("catalog_page", { page: p });
      games = archive.games;
      totalPages = archive.meta.total_pages;
      page = archive.meta.page;
      status = "ready";
    } catch (err) {
      status = "error";
      error = String(err);
    }
  }

  onMount(() => load(page));

  function next() {
    if (totalPages === null || page < totalPages) load(page + 1);
  }

  function prev() {
    if (page > 1) load(page - 1);
  }

  function platformLabel(p: string): string {
    switch (p) {
      case "pc":
        return "PC";
      case "android":
        return "Android";
      case "mac":
        return "Mac";
      case "linux":
        return "Linux";
      default:
        return p;
    }
  }
</script>

{#if status === "loading"}
  <p class="note">Loading catalog…</p>
{:else if status === "error"}
  <p class="note error">{error}</p>
{:else}
  <div class="bar">
    <span class="bar-title" aria-label="Game catalog">
      {games.length} games{games.length > 0 ? ` · page ${page}` : ""}{totalPages
        ? ` of ${totalPages}`
        : ""}
    </span>
    <div class="pager">
      <button class="mini" disabled={page <= 1} onclick={prev} aria-label="Previous page">
        ‹
      </button>
      <button
        class="mini"
        disabled={totalPages !== null && page >= totalPages}
        onclick={next}
        aria-label="Next page"
      >
        ›
      </button>
    </div>
  </div>
  {#if games.length === 0}
    <p class="note">No games found on this page.</p>
  {:else}
    <div class="grid" role="list">
      {#each games as game (game.slug)}
        <div class="tile" role="listitem" title={game.title}>
          <div class="cover" aria-hidden="true">
            {#if game.thumb_url}
              <img src={game.thumb_url} alt="" loading="lazy" />
            {/if}
          </div>
          <div class="tile-title">{game.title}</div>
          <div class="tile-meta">
            {#each game.platforms as p, i (p)}{i > 0 ? " · " : ""}{platformLabel(p)}{/each}
            {game.state ? ` · ${game.state}` : ""}
          </div>
        </div>
      {/each}
    </div>
  {/if}
{/if}

<style>
  .bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: calc(var(--lz-gap) * 0.6) var(--lz-gap) 0;
  }

  .bar-title {
    color: var(--lz-text-dim);
    font-size: 13px;
  }

  .pager {
    display: flex;
    gap: 6px;
  }

  .mini {
    background: var(--lz-surface);
    color: var(--lz-text);
    border: 1px solid var(--lz-surface-2);
    border-radius: var(--lz-radius);
    width: 28px;
    height: 28px;
    cursor: pointer;
  }

  .mini:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: var(--lz-gap);
    padding: var(--lz-gap);
  }

  .tile {
    cursor: pointer;
    border-radius: var(--lz-radius);
    overflow: hidden;
    background: var(--lz-surface);
    transition: transform 0.12s ease;
  }

  .tile:hover {
    transform: translateY(-2px);
  }

  .cover {
    aspect-ratio: 16 / 9;
    background:
      linear-gradient(135deg, var(--lz-primary), var(--lz-cyan));
    opacity: 0.9;
  }

  .cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .tile-title {
    padding: 6px 8px 0;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tile-meta {
    padding: 0 8px 8px;
    color: var(--lz-text-dim);
    font-size: 12px;
  }

  .note {
    padding: var(--lz-gap);
    color: var(--lz-text-dim);
  }

  .note.error {
    color: var(--lz-danger);
  }
</style>