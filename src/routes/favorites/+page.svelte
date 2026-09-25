<svelte:options runes={true} />

<script lang="ts">
  import { onMount } from "svelte";
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";
  import { favoritesStore } from "$lib/stores/clientCache";

  type LoadState = "loading" | "ready" | "error";

  interface GameCard {
    slug: string;
    post_id: number | null;
    title: string;
    thumb_url: string | null;
    platforms: string[];
    engine: string | null;
    state: string | null;
    version_tag: string | null;
    developer: string | null;
    genres: string[];
    genre_slugs: string[];
    external_genres: string[];
    views: number | null;
  }

  let status: LoadState = $state(favoritesStore.isLoaded ? "ready" : "loading");
  let error = $state("");
  let games: GameCard[] = $state(favoritesStore.games);
  let coverUrls: Record<string, string | null> = $state({ ...favoritesStore.coverUrls });
  let togglingFavorite = $state<Record<string, boolean>>({});
  let isRefreshing = $state(false);

  const platformLabel: Record<string, string> = {
    pc: "PC",
    mac: "Mac",
    linux: "Linux",
  };

  function formatArtworkUrl(url: string | null | undefined): string | null {
    if (!url) return null;
    let clean = url.trim();
    if (clean.startsWith("[") && clean.endsWith("]")) {
      try {
        const arr = JSON.parse(clean);
        if (Array.isArray(arr) && arr.length > 0 && typeof arr[0] === "string") {
          clean = arr[0].trim();
        }
      } catch {}
    }
    if (
      clean.startsWith("http://") ||
      clean.startsWith("https://") ||
      clean.startsWith("data:") ||
      clean.startsWith("asset:")
    ) {
      return clean;
    }
    try {
      const cleanPath = clean.startsWith("file://")
        ? decodeURIComponent(clean.replace(/^file:\/\/\/?/, ""))
        : clean;
      return convertFileSrc(cleanPath);
    } catch {
      return clean;
    }
  }

  async function load(force = false) {
    if (force) {
      isRefreshing = true;
    } else if (!favoritesStore.isLoaded) {
      status = "loading";
    }
    error = "";
    try {
      const rows = await invoke<GameCard[]>("favorites_list");
      games = rows;
      await loadArtwork(rows);
      favoritesStore.games = games;
      favoritesStore.coverUrls = { ...coverUrls };
      favoritesStore.isLoaded = true;
      status = "ready";
    } catch (err) {
      if (!favoritesStore.isLoaded) {
        status = "error";
      }
      error = String(err);
    } finally {
      isRefreshing = false;
    }
  }

  async function loadArtwork(list: GameCard[]) {
    for (const game of list) {
      if (game.thumb_url && !coverUrls[game.slug]) {
        coverUrls[game.slug] = formatArtworkUrl(game.thumb_url);
      }
    }
    await Promise.all(
      list.map(async (game) => {
        try {
          const url = await invoke<string | null>("artwork_url", {
            card: {
              slug: game.slug,
              post_id: game.post_id,
              title: game.title,
              thumb_url: game.thumb_url ?? null,
            },
            kind: "cover",
          });
          const formatted = formatArtworkUrl(url ?? game.thumb_url ?? null);
          if (formatted) {
            coverUrls[game.slug] = formatted;
          }
        } catch {
          const formatted = formatArtworkUrl(game.thumb_url ?? null);
          if (formatted) {
            coverUrls[game.slug] = formatted;
          }
        }
      }),
    );
  }

  async function removeFavorite(game: GameCard) {
    if (togglingFavorite[game.slug]) return;
    togglingFavorite[game.slug] = true;
    try {
      await invoke("favorite_remove", { slug: game.slug });
      games = games.filter((g) => g.slug !== game.slug);
      favoritesStore.games = games;
      window.dispatchEvent(new CustomEvent("favorites-changed"));
    } catch (err) {
      error = String(err);
    } finally {
      togglingFavorite[game.slug] = false;
    }
  }

  onMount(() => {
    if (!favoritesStore.isLoaded) {
      load();
    }
    const handler = () => load(false);
    window.addEventListener("favorites-changed", handler);
    return () => window.removeEventListener("favorites-changed", handler);
  });
</script>

{#if status === "loading" && games.length === 0}
  <p class="note">Loading favorites…</p>
{:else if status === "error" && games.length === 0}
  <p class="note error">{error}</p>
{:else}
  <section class="favorites">
    <div class="head">
      <h1>
        <span class="head-icon" aria-hidden="true"
          ><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20.5 4.6 13a4.8 4.8 0 0 1 0-6.9 5.1 5.1 0 0 1 7.4 0l.6.6.6-.6a5.1 5.1 0 0 1 7.4 0 4.8 4.8 0 0 1 0 6.9L12 20.5Z"/></svg></span
        >Favorites
      </h1>
      <button
        type="button"
        class="refresh-btn"
        class:spinning={isRefreshing}
        disabled={isRefreshing}
        onclick={() => load(true)}
        title="Refresh favorites"
        aria-label="Refresh favorites"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67"/>
        </svg>
        <span>{isRefreshing ? "Refreshing…" : "Refresh"}</span>
      </button>
    </div>

    {#if games.length === 0}
      <p class="note">
        Games you favorite in the Library will appear here. Use the heart on any
        installed game tile to save it for quick access.
      </p>
      <div class="empty" role="list">
        <div class="empty-card" role="listitem">No favorites yet.</div>
      </div>
    {:else}
      <div class="grid" role="list">
        {#each games as game (game.slug)}
          <div class="tile" role="listitem">
            <div class="tile-cover">
              <button
                class="cover-btn"
                onclick={() => goto(`/library/${game.slug}`)}
                aria-label={`Open details for ${game.title}`}
              >
                <div class="cover">
                  {#if coverUrls[game.slug]}
                    <img src={coverUrls[game.slug]} alt="" loading="lazy" />
                  {:else}
                    <span class="cover-glyph"
                      ><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="6.5" width="18" height="11" rx="5.5"/><circle cx="8" cy="11.5" r="1.1" fill="currentColor" stroke="none"/><circle cx="12.5" cy="11.5" r="1.1" fill="currentColor" stroke="none"/><path d="M16.2 14.4h.01M18.6 12.4h.01"/></svg></span
                    >
                  {/if}
                </div>
              </button>
              <button
                class="heart-btn filled"
                onclick={(e) => {
                  e.stopPropagation();
                  removeFavorite(game);
                }}
                disabled={togglingFavorite[game.slug]}
                aria-label={`Remove ${game.title} from favorites`}
                title="Remove favorite"
              >
                <svg viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M12 20.5 4.6 13a4.8 4.8 0 0 1 0-6.9 5.1 5.1 0 0 1 7.4 0l.6.6.6-.6a5.1 5.1 0 0 1 7.4 0 4.8 4.8 0 0 1 0 6.9L12 20.5Z"/>
                </svg>
              </button>
            </div>
            <button
              class="tile-title-btn"
              onclick={() => goto(`/library/${game.slug}`)}
              aria-label={`Open details for ${game.title}`}
            >
              {game.title}
            </button>
            <div class="tile-meta">
              {#each game.platforms.slice(0, 3) as p}
                <span>{platformLabel[p.toLowerCase()] ?? p}</span>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </section>
{/if}

<style>
  .favorites {
    padding: var(--lz-gap);
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--lz-gap);
  }

  .favorites h1 {
    font-size: 18px;
    margin: 0;
    display: flex;
    align-items: center;
    gap: 8px;
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

  .head-icon :global(svg) {
    width: 22px;
    height: 22px;
    color: var(--lz-cyan);
    display: block;
  }

  .note {
    color: var(--lz-text-dim);
  }

  .note.error {
    color: var(--lz-danger);
  }

  .empty {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: var(--lz-gap);
    margin-top: var(--lz-gap);
  }

  .empty-card {
    aspect-ratio: 2 / 3;
    border-radius: var(--lz-radius);
    background: var(--lz-surface);
    border: 1px dashed var(--lz-surface-2);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--lz-text-dim);
    font-size: 12px;
    text-align: center;
    padding: 8px;
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
  }

  .tile-cover {
    position: relative;
  }

  .cover-btn {
    all: unset;
    display: block;
    width: 100%;
    cursor: pointer;
  }

  .cover {
    aspect-ratio: 2 / 3;
    background:
      linear-gradient(135deg, var(--lz-primary), var(--lz-cyan));
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  .cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .cover-glyph {
    color: var(--lz-bg);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .cover-glyph :global(svg) {
    width: 44px;
    height: 44px;
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
    color: var(--lz-danger);
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

  .heart-btn :global(svg) {
    width: 16px;
    height: 16px;
  }

  .tile-title-btn {
    all: unset;
    padding: 8px 8px 0;
    font-weight: 600;
    font-size: 12px;
    line-height: 1.2;
    line-clamp: 2;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    cursor: pointer;
  }

  .tile-meta {
    padding: 4px 8px 8px;
    display: flex;
    gap: 6px;
    color: var(--lz-text-dim);
    font-size: 11px;
    flex-wrap: wrap;
  }
</style>
