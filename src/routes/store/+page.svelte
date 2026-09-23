<svelte:options runes={true} />

<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";

  type LoadState = "loading" | "ready" | "error";

  interface GameCard {
    slug: string;
    title: string;
    thumb_url: string | null;
    platforms: string[];
    engine: string | null;
    state: string | null;
    version_tag: string | null;
    developer: string | null;
    genres: string[];
    genre_slugs: string[];
    views: string | null;
  }

  interface ArchiveFilter {
    q?: string;
    platform?: string;
    engine?: string;
    state?: string;
    sort?: string;
    include_tags?: string[];
    exclude_tags?: string[];
  }

  interface ArchiveMeta {
    page: number;
    total_pages: number | null;
  }

  interface ArchivePage {
    games: GameCard[];
    meta: ArchiveMeta;
  }

  interface Genre {
    label: string;
    slug: string;
    count: number | null;
  }

  const PLATFORMS = ["", "PC", "Mac", "Linux", "Android"];
  const ENGINES = [
    "",
    "RenPy",
    "RPG Maker",
    "Unity",
    "Unreal Engine",
    "HTML",
    "Flash",
    "Wolf RPG",
    "Other",
  ];
  const STATES = ["", "Finished", "Ongoing", "Abandoned", "Onhold", "Demo"];
  const SORTS = [
    "",
    "Last Update",
    "Popularity",
    "New to Old",
    "Old to New",
    "Rating",
  ];

  const platformLabel: Record<string, string> = {
    pc: "PC",
    android: "Android",
    mac: "Mac",
    linux: "Linux",
  };

  let status: LoadState = $state("loading");
  let error = $state("");
  let page = $state(1);
  let totalPages: number | null = $state(null);
  let games: GameCard[] = $state([]);
  let genres: Genre[] = $state([]);

  let q = $state("");
  let platform = $state("");
  let engine = $state("");
  let devState = $state("");
  let sort = $state("Popularity");
  let includeTags = $state<string[]>([]);
  let excludeTags = $state<string[]>([]);

  const appliedCount = $derived(
    (q ? 1 : 0) +
      (platform ? 1 : 0) +
      (engine ? 1 : 0) +
      (devState ? 1 : 0) +
      (sort ? 1 : 0) +
      includeTags.length +
      excludeTags.length,
  );

  function buildFilter(): ArchiveFilter {
    return {
      q: q || undefined,
      platform: platform || undefined,
      engine: engine || undefined,
      state: devState || undefined,
      sort: sort || undefined,
      include_tags: includeTags.length ? includeTags : undefined,
      exclude_tags: excludeTags.length ? excludeTags : undefined,
    };
  }

  async function load(p: number) {
    status = "loading";
    try {
      const archive = await invoke<ArchivePage>("catalog_page", {
        page: p,
        filter: buildFilter(),
      });
      games = archive.games;
      totalPages = archive.meta.total_pages;
      page = archive.meta.page;
      status = "ready";
    } catch (err) {
      status = "error";
      error = String(err);
    }
  }

  onMount(async () => {
    load(page);
    try {
      genres = await invoke<Genre[]>("catalog_genres");
    } catch {
      genres = [];
    }
  });

  function toggleInclude(slug: string) {
    const had = includeTags.includes(slug);
    includeTags = had ? includeTags.filter((s) => s !== slug) : [...includeTags, slug];
    excludeTags = excludeTags.filter((s) => s !== slug);
    page = 1;
    load(page);
  }

  function toggleExclude(slug: string) {
    const had = excludeTags.includes(slug);
    excludeTags = had ? excludeTags.filter((s) => s !== slug) : [...excludeTags, slug];
    includeTags = includeTags.filter((s) => s !== slug);
    page = 1;
    load(page);
  }

  function applySelect() {
    page = 1;
    load(page);
  }

  function submitSearch() {
    applySelect();
  }

  function clearFilters() {
    q = "";
    platform = "";
    engine = "";
    devState = "";
    sort = "Popularity";
    includeTags = [];
    excludeTags = [];
    page = 1;
    load(page);
  }

  function next() {
    if (totalPages === null || page < totalPages) load(page + 1);
  }

  function prev() {
    if (page > 1) load(page - 1);
  }

  function openGame(slug: string) {
    goto(`/store/${slug}`);
  }
</script>

<div class="store">
  {#if status === "loading"}
    <p class="note">Loading catalog…</p>
  {:else if status === "error"}
    <p class="note error">{error}</p>
  {:else}
    <aside class="rail" aria-label="Store categories">
      <div class="rail-group">
        <h2 class="rail-title">Sort By</h2>
        <select aria-label="Sort" bind:value={sort} onchange={applySelect}>
          {#each SORTS as s (s)}
            <option value={s}>{s || "Default"}</option>
          {/each}
        </select>
      </div>

      <div class="rail-group">
        <h2 class="rail-title">Platform</h2>
        <div class="rail-list">
          {#each PLATFORMS as label (label)}
            <button
              class="rail-item"
              class:on={platform === label}
              onclick={() => {
                platform = label;
                applySelect();
              }}
            >
              {label || "All Platforms"}
            </button>
          {/each}
        </div>
      </div>

      <div class="rail-group">
        <h2 class="rail-title">Engine</h2>
        <select aria-label="Engine" bind:value={engine} onchange={applySelect}>
          {#each ENGINES as e (e)}
            <option value={e}>{e || "All"}</option>
          {/each}
        </select>
      </div>

      <div class="rail-group">
        <h2 class="rail-title">State</h2>
        <select aria-label="State" bind:value={devState} onchange={applySelect}>
          {#each STATES as s (s)}
            <option value={s}>{s || "All"}</option>
          {/each}
        </select>
      </div>

      <div class="rail-group tags">
        <h2 class="rail-title">Tags</h2>
        <div class="tag-lists">
          {#each genres as genre (genre.slug)}
            <div class="tag-row">
              <span class="tag-label" title={genre.label}>{genre.label}</span>
              <div class="tag-btns">
                <button
                  class="tag-btn inc"
                  class:on={includeTags.includes(genre.slug)}
                  onclick={() => toggleInclude(genre.slug)}
                  aria-label={`Include ${genre.label}`}
                >
                  +
                </button>
                <button
                  class="tag-btn exc"
                  class:on={excludeTags.includes(genre.slug)}
                  onclick={() => toggleExclude(genre.slug)}
                  aria-label={`Exclude ${genre.label}`}
                >
                  –
                </button>
              </div>
            </div>
          {/each}
          {#if genres.length === 0}
            <span class="tag-empty">Genre list unavailable.</span>
          {/if}
        </div>
      </div>
    </aside>

    <section class="main-col">
      <header class="store-bar">
        <form
          onsubmit={(e) => {
            e.preventDefault();
            submitSearch();
          }}
        >
          <input
            class="search"
            type="search"
            placeholder="Search…"
            bind:value={q}
            aria-label="Search games"
          />
        </form>
        <div class="bar-stats" aria-label="Game catalog">
          {games.length} games · page {page}{totalPages ? ` of ${totalPages}` : ""}
        </div>
        {#if appliedCount > 0}
          <button class="clear" onclick={clearFilters} aria-label="Clear all filters">
            Clear ({appliedCount})
          </button>
        {/if}
      </header>

      {#if games.length === 0}
        <p class="note">No games match these filters.</p>
      {:else}
        <div class="grid" role="list">
          {#each games as game (game.slug)}
            <div class="tile" role="listitem">
              <button
                class="tile-btn"
                onclick={() => openGame(game.slug)}
                title={game.title}
                aria-label={game.title}
              >
                <div class="cover" aria-hidden="true">
                  {#if game.thumb_url}
                    <img src={game.thumb_url} alt="" loading="lazy" />
                  {/if}
                </div>
                <div class="tile-title">{game.title}</div>
                <div class="tile-meta">
                  {#each game.platforms as p, i (p)}
                    {i > 0 ? " · " : ""}{platformLabel[p] ?? p}
                  {/each}
                  {game.state ? ` · ${game.state}` : ""}
                </div>
              </button>
            </div>
          {/each}
        </div>
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
      {/if}
    </section>
  {/if}
</div>

<style>
  .store {
    display: flex;
    gap: 0;
    min-height: 100%;
  }

  .rail {
    width: 240px;
    flex: 0 0 auto;
    padding: var(--lz-gap);
    border-right: 1px solid var(--lz-surface-2);
    background: var(--lz-glass);
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--lz-surface-2) transparent;
    max-height: 100vh;
  }

  .rail-group {
    margin-bottom: 18px;
  }

  .rail-title {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--lz-text-dim);
    margin: 0 0 6px;
  }

  .rail-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .rail-item {
    appearance: none;
    border: none;
    background: transparent;
    color: var(--lz-text);
    font: inherit;
    text-align: left;
    padding: 4px 8px;
    border-radius: var(--lz-radius);
    cursor: pointer;
  }

  .rail-item:hover {
    background: var(--lz-surface-2);
  }

  .rail-item.on {
    color: var(--lz-cyan);
    background: var(--lz-surface-2);
  }

  select {
    width: 100%;
    background: var(--lz-surface);
    color: var(--lz-text);
    border: 1px solid var(--lz-surface-2);
    border-radius: var(--lz-radius);
    font: inherit;
    padding: 4px 6px;
  }

  .tag-lists {
    max-height: 320px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .tag-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    padding: 1px 0;
  }

  .tag-label {
    font-size: 12px;
    color: var(--lz-text-dim);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tag-btns {
    display: flex;
    gap: 3px;
    flex: 0 0 auto;
  }

  .tag-btn {
    width: 20px;
    height: 20px;
    border: 1px solid var(--lz-surface-2);
    background: var(--lz-surface);
    color: var(--lz-text-dim);
    border-radius: 3px;
    cursor: pointer;
    font: inherit;
    line-height: 1;
  }

  .tag-btn.inc.on {
    border-color: var(--lz-ok);
    color: var(--lz-ok);
  }

  .tag-btn.exc.on {
    border-color: var(--lz-danger);
    color: var(--lz-danger);
  }

  .tag-empty {
    color: var(--lz-text-dim);
    font-size: 12px;
  }

  .main-col {
    flex: 1 1 auto;
    min-width: 0;
    padding: var(--lz-gap);
  }

  .store-bar {
    display: flex;
    align-items: center;
    gap: var(--lz-gap);
    margin-bottom: var(--lz-gap);
  }

  .search {
    width: 320px;
    border-radius: 8px;
    border: 1px solid var(--lz-surface-2);
    background: var(--lz-glass);
    color: var(--lz-text);
    font: inherit;
    padding: 8px 12px;
  }

  .search::placeholder {
    color: var(--lz-text-dim);
  }

  .bar-stats {
    color: var(--lz-text-dim);
    font-size: 13px;
  }

  .clear {
    appearance: none;
    border: 1px solid var(--lz-surface-2);
    background: var(--lz-surface);
    color: var(--lz-text);
    font: inherit;
    font-size: 13px;
    padding: 6px 10px;
    border-radius: var(--lz-radius);
    cursor: pointer;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
    gap: var(--lz-gap);
  }

  .tile {
    border-radius: var(--lz-radius);
    overflow: hidden;
  }

  .tile-btn {
    appearance: none;
    border: none;
    background: transparent;
    color: var(--lz-text);
    font: inherit;
    text-align: center;
    cursor: pointer;
    padding: 0;
    display: flex;
    flex-direction: column;
    border-radius: var(--lz-radius);
    overflow: hidden;
    transition: transform 0.12s ease;
    width: 100%;
  }

  .tile-btn:hover {
    transform: translateY(-2px);
  }

  .cover {
    aspect-ratio: 2 / 3;
    background:
      linear-gradient(135deg, var(--lz-primary), var(--lz-cyan));
    border-radius: var(--lz-radius);
    overflow: hidden;
  }

  .cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .tile-title {
    margin-top: 4px;
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
    color: var(--lz-text-dim);
    font-size: 11px;
    margin-top: 2px;
  }

  .pager {
    display: flex;
    justify-content: center;
    gap: 6px;
    margin-top: var(--lz-gap);
  }

  .mini {
    background: var(--lz-surface);
    color: var(--lz-text);
    border: 1px solid var(--lz-surface-2);
    border-radius: var(--lz-radius);
    width: 28px;
    height: 28px;
    cursor: pointer;
    font: inherit;
  }

  .mini:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .note {
    color: var(--lz-text-dim);
  }

  .note.error {
    color: var(--lz-danger);
  }
</style>