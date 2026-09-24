<svelte:options runes={true} />

<script lang="ts">
  import { onMount } from "svelte";
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import MediaCarousel from "$lib/components/MediaCarousel.svelte";

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
    thumb_url?: string | null;
  }

  interface GameData {
    slug: string;
    post_id: number | null;
    title: string;
    developer: string | null;
    current_version: string | null;
    engine: string | null;
    platforms: string[];
    genres: string[];
    size_label: string | null;
    censorship: string | null;
    screenshots: string[];
    description: string | null;
    rating?: number | null;
  }

  interface Enrichment {
    description?: string | null;
    developer?: string | null;
    rating?: number | null;
    tags: string[];
    genres: string[];
    screenshots: string[];
  }

  const slug = $derived(
    (page.params?.slug ?? (page.url.pathname.match(/\/library\/([^/?#]+)/) ?? [])[1] ?? "")
      .replace(/\/+$/, "")
  );

  let status: LoadState = $state("loading");
  let error = $state("");

  let installed = $state<LibraryGame | null>(null);
  let game = $state<GameData | null>(null);
  let isFavorite = $state(false);

  let launching = $state(false);
  let manageOpen = $state(false);
  let uninstallConfirm = $state(false);
  let uninstalling = $state(false);
  let toastMsg = $state<string | null>(null);

  // Artwork
  let heroArtUrl = $state<string | null>(null);
  let coverArtUrl = $state<string | null>(null);

  function formatArtworkUrl(raw: string | null | undefined): string | null {
    if (!raw) return null;
    if (raw.startsWith("http://") || raw.startsWith("https://") || raw.startsWith("data:")) {
      return raw;
    }
    try {
      return convertFileSrc(raw);
    } catch {
      return raw;
    }
  }

  function formatPlaytime(seconds: number | undefined): string {
    if (!seconds || seconds <= 0) return "Unplayed";
    if (seconds < 60) return `${seconds}s`;
    const mins = Math.floor(seconds / 60);
    if (mins < 60) return `${mins}m`;
    const hours = (seconds / 3600).toFixed(1);
    return `${hours} hrs`;
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
      pc: "Windows PC",
      windows: "Windows PC",
      mac: "Mac OS",
      linux: "Linux",
      android: "Android",
    };
    return map[p.toLowerCase()] ?? p;
  }

  async function load(currentSlug: string) {
    status = "loading";
    error = "";
    try {
      // 1. Load library manifests
      const listing = await invoke<{ root: string | null; games: LibraryGame[] }>("library_list");
      const found = listing.games.find((g) => g.slug === currentSlug);
      installed = found ?? null;

      // 2. Load favorites
      try {
        const favs = await invoke<{ slug: string }[]>("favorites_list");
        isFavorite = favs.some((f) => f.slug === currentSlug);
      } catch {
        isFavorite = false;
      }

      // 3. Load scraped game metadata from memory/catalog
      try {
        const scraped = await invoke<GameData>("game_page", { slug: currentSlug });
        game = {
          ...scraped,
          screenshots: scraped.screenshots ?? [],
          genres: scraped.genres ?? [],
        };
      } catch {
        // Fallback to installed manifest information if scraped page is unavailable
        if (found) {
          game = {
            slug: found.slug,
            post_id: found.post_id,
            title: found.title,
            developer: null,
            current_version: found.version,
            engine: found.engine,
            platforms: [found.platform],
            genres: [],
            size_label: formatSize(found.size_on_disk),
            censorship: null,
            screenshots: found.thumb_url ? [found.thumb_url] : [],
            description: null,
          };
        }
      }

      if (found?.thumb_url) {
        coverArtUrl = formatArtworkUrl(found.thumb_url);
      }

      status = "ready";

      // 4. SteamGridDB & IGDB external enrichment to improve metadata & artwork
      if (game) {
        enrichGame(game);
      }
    } catch (err) {
      status = "error";
      error = String(err);
    }
  }

  async function enrichGame(g: GameData) {
    try {
      const [enrichment, bgArt, coverArt] = await Promise.all([
        invoke<Enrichment>("content_enrich", {
          card: {
            slug: g.slug,
            post_id: g.post_id,
            title: g.title,
            developer: g.developer,
            genres: g.genres ?? [],
            external_genres: [],
            description: g.description,
            thumb_url: g.screenshots?.[0] ?? null,
          },
        }).catch(() => null),
        invoke<string | null>("artwork_url", {
          card: {
            slug: g.slug,
            post_id: g.post_id,
            title: g.title,
            thumb_url: g.screenshots?.[0] ?? null,
          },
          kind: "background",
        }).catch(() => null),
        invoke<string | null>("artwork_url", {
          card: {
            slug: g.slug,
            post_id: g.post_id,
            title: g.title,
            thumb_url: g.screenshots?.[0] ?? null,
          },
          kind: "cover",
        }).catch(() => null),
      ]);

      if (bgArt) {
        heroArtUrl = formatArtworkUrl(bgArt);
      }
      if (coverArt && !coverArtUrl) {
        coverArtUrl = formatArtworkUrl(coverArt);
      }

      if (enrichment && game && game.slug === g.slug) {
        if ((!game.description || game.description.trim().length === 0) && enrichment.description) {
          game.description = enrichment.description;
        }
        if ((!game.developer || game.developer.trim().length === 0) && enrichment.developer) {
          game.developer = enrichment.developer;
        }
        if ((!game.rating || game.rating === 0) && enrichment.rating) {
          game.rating = enrichment.rating;
        }
        if (enrichment.genres) {
          for (const item of enrichment.genres) {
            if (!game.genres.includes(item)) {
              game.genres.push(item);
            }
          }
        }
        if (enrichment.tags) {
          for (const item of enrichment.tags) {
            if (!game.genres.includes(item)) {
              game.genres.push(item);
            }
          }
        }
        if (enrichment.screenshots) {
          for (const s of enrichment.screenshots) {
            if (!game.screenshots.includes(s) && game.screenshots.length < 10) {
              game.screenshots.push(s);
            }
          }
          if (game.screenshots.length > 10) {
            game.screenshots = game.screenshots.slice(0, 10);
          }
        }
      }
    } catch (e) {
      console.warn("Enrichment note:", e);
    }
  }

  async function launchGame() {
    if (!slug) return;
    launching = true;
    try {
      await invoke("game_launch", { slug });
      toastMsg = `Launched ${game?.title || slug}`;
      const listing = await invoke<{ root: string | null; games: LibraryGame[] }>("library_list");
      installed = listing.games.find((g) => g.slug === slug) ?? installed;
    } catch (err) {
      toastMsg = `Launch failed: ${err}`;
    } finally {
      launching = false;
    }
  }

  async function openFolder() {
    manageOpen = false;
    try {
      await invoke("open_game_folder", { slug });
    } catch (err) {
      toastMsg = `Could not open directory: ${err}`;
    }
  }

  async function makeShortcut() {
    manageOpen = false;
    try {
      await invoke("create_shortcut", { slug });
      toastMsg = `Created shortcut for ${game?.title || slug}`;
    } catch (err) {
      toastMsg = `Failed to create shortcut: ${err}`;
    }
  }

  async function toggleFavorite() {
    manageOpen = false;
    try {
      if (isFavorite) {
        await invoke("favorite_remove", { slug });
        isFavorite = false;
        toastMsg = `Removed from favorites`;
      } else {
        await invoke("favorite_add", { slug });
        isFavorite = true;
        toastMsg = `Added to favorites`;
      }
      window.dispatchEvent(new CustomEvent("favorites-changed"));
    } catch (err) {
      toastMsg = `Favorite error: ${err}`;
    }
  }

  async function uninstallGame() {
    uninstalling = true;
    try {
      await invoke("uninstall_game", { slug });
      toastMsg = `Uninstalled ${game?.title || slug}`;
      uninstallConfirm = false;
      manageOpen = false;
      setTimeout(() => goto("/library"), 1200);
    } catch (err) {
      toastMsg = `Uninstall failed: ${err}`;
    } finally {
      uninstalling = false;
    }
  }

  function back() {
    if (typeof window !== "undefined" && window.history.length > 1) {
      window.history.back();
    } else {
      goto("/library");
    }
  }

  onMount(() => {
    if (slug) {
      load(slug);
    }
  });
</script>

<div class="installed-page">
  {#if status === "loading"}
    <p class="note">Loading game details…</p>
  {:else if status === "error"}
    <p class="note error">{error}</p>
  {:else if game}
    <!-- Hero Banner with Artwork & Title -->
    <div class="hero">
      {#if heroArtUrl}
        <img class="hero-img" src={heroArtUrl} alt="" aria-hidden="true" />
      {:else if game.screenshots.length > 0}
        <img class="hero-img" src={game.screenshots[0]} alt="" aria-hidden="true" />
      {/if}
      <div class="hero-overlay"></div>

      <button class="back-btn" onclick={back}>
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M15 5l-7 7 7 7"/></svg>
        Back to Library
      </button>

      <div class="hero-meta">
        <h1>{game.title}</h1>
        <div class="meta-row">
          {#if game.developer}<span>by {game.developer}</span>{/if}
          {#if game.current_version}<span>v{game.current_version}</span>{/if}
          {#if game.engine}<span>{game.engine}</span>{/if}
          {#if game.censorship}<span>{game.censorship}</span>{/if}
          {#if typeof game.rating === "number" && game.rating > 0}
            <span class="rating">★ {game.rating.toFixed(1)}</span>
          {/if}
        </div>
        {#if game.genres.length > 0}
          <div class="genre-row">
            {#each game.genres as genre (genre)}
              <span class="genre-chip">{genre}</span>
            {/each}
          </div>
        {/if}
      </div>
    </div>

    <!-- Actions & Launch Bar -->
    <div class="action-bar">
      <div class="action-left">
        <button
          class="play-btn"
          onclick={launchGame}
          disabled={launching}
          aria-label={`Play ${game.title}`}
        >
          {#if launching}
            <span class="play-icon" aria-hidden="true">⏳</span>
            <span>Launching…</span>
          {:else}
            <span class="play-icon" aria-hidden="true">▶</span>
            <span>Play</span>
          {/if}
        </button>

        <div class="manage-dropdown">
          <button
            class="manage-btn"
            onclick={() => (manageOpen = !manageOpen)}
            aria-haspopup="menu"
            aria-expanded={manageOpen}
          >
            <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
            <span>Manage</span>
            <span class="arrow" aria-hidden="true">▾</span>
          </button>

          {#if manageOpen}
            <div class="menu-popover" role="menu">
              <button class="pop-item" role="menuitem" onclick={openFolder}>
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
                Browse Local Files
              </button>
              <button class="pop-item" role="menuitem" onclick={makeShortcut}>
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
                Create Shortcut
              </button>
              <button class="pop-item" role="menuitem" onclick={toggleFavorite}>
                <svg viewBox="0 0 24 24" width="14" height="14" fill={isFavorite ? "currentColor" : "none"} stroke="currentColor" stroke-width="2"><path d="M12 20.5 4.6 13a4.8 4.8 0 0 1 0-6.9 5.1 5.1 0 0 1 7.4 0l.6.6.6-.6a5.1 5.1 0 0 1 7.4 0 4.8 4.8 0 0 1 0 6.9L12 20.5Z"/></svg>
                {isFavorite ? "Remove from Favorites" : "Add to Favorites"}
              </button>
              <button class="pop-item" role="menuitem" onclick={() => goto(`/store/${slug}`)}>
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><circle cx="9" cy="21" r="1"/><circle cx="20" cy="21" r="1"/><path d="M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6"/></svg>
                View in Store
              </button>
              <hr class="pop-divider" />
              <button
                class="pop-item danger"
                role="menuitem"
                onclick={() => {
                  manageOpen = false;
                  uninstallConfirm = true;
                }}
              >
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                Uninstall Game…
              </button>
            </div>
          {/if}
        </div>

        <button
          class="heart-btn"
          class:filled={isFavorite}
          onclick={toggleFavorite}
          aria-label={isFavorite ? "Remove from favorites" : "Add to favorites"}
          title={isFavorite ? "Remove from favorites" : "Add to favorites"}
        >
          <svg viewBox="0 0 24 24" width="18" height="18" fill={isFavorite ? "currentColor" : "none"} stroke="currentColor" stroke-width="1.8">
            <path d="M12 20.5 4.6 13a4.8 4.8 0 0 1 0-6.9 5.1 5.1 0 0 1 7.4 0l.6.6.6-.6a5.1 5.1 0 0 1 7.4 0 4.8 4.8 0 0 1 0 6.9L12 20.5Z"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- Uninstall Confirmation Bar -->
    {#if uninstallConfirm}
      <div class="confirm-box" role="alert">
        <div class="confirm-content">
          <strong>Uninstall {game.title}?</strong>
          <span>This will remove all installed files from your library disk.</span>
        </div>
        <div class="confirm-btns">
          <button class="btn danger" onclick={uninstallGame} disabled={uninstalling}>
            {uninstalling ? "Uninstalling…" : "Uninstall"}
          </button>
          <button class="btn secondary" onclick={() => (uninstallConfirm = false)} disabled={uninstalling}>
            Cancel
          </button>
        </div>
      </div>
    {/if}

    <!-- Toast Feedback -->
    {#if toastMsg}
      <div class="toast-box">
        <span>{toastMsg}</span>
        <button class="toast-close" onclick={() => (toastMsg = null)}>✕</button>
      </div>
    {/if}

    <!-- Playtime & Statistics Section -->
    {#if installed}
      <div class="stats-grid">
        <div class="stat-card">
          <span class="stat-title">Playtime</span>
          <span class="stat-num glow">{formatPlaytime(installed.playtime_seconds)}</span>
        </div>
        <div class="stat-card">
          <span class="stat-title">Last Played</span>
          <span class="stat-num">{formatLastPlayed(installed.last_played_at)}</span>
        </div>
        <div class="stat-card">
          <span class="stat-title">Sessions</span>
          <span class="stat-num">{installed.play_count ?? 0}</span>
        </div>
        <div class="stat-card">
          <span class="stat-title">Size on Disk</span>
          <span class="stat-num">{formatSize(installed.size_on_disk)}</span>
        </div>
      </div>
    {/if}

    <!-- Game Description -->
    {#if game.description}
      <section class="section-card">
        <h2>About This Game</h2>
        <p class="description-text">{game.description}</p>
      </section>
    {/if}

    <!-- Artwork & Media Carousel (Up to 10 Images) -->
    <MediaCarousel title={game.title} screenshots={game.screenshots} />

    <!-- Installation Metadata & Path Details -->
    {#if installed}
      <section class="section-card meta-details">
        <h2>Installation Details</h2>
        <div class="details-list">
          <div class="detail-row">
            <span class="detail-label">Platform:</span>
            <span class="detail-val">{platformLabel(installed.platform)}</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Installed Version:</span>
            <span class="detail-val">{installed.version}</span>
          </div>
          {#if installed.engine}
            <div class="detail-row">
              <span class="detail-label">Game Engine:</span>
              <span class="detail-val">{installed.engine}</span>
            </div>
          {/if}
          <div class="detail-row">
            <span class="detail-label">Install Location:</span>
            <code class="detail-path">{installed.install_path}</code>
          </div>
        </div>
      </section>
    {/if}
  {/if}
</div>

<style>
  .installed-page {
    padding: var(--lz-gap);
    max-width: 960px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: var(--lz-gap);
  }

  .note {
    color: var(--lz-text-dim);
    font-size: 14px;
    padding: 24px;
    text-align: center;
  }

  .note.error {
    color: var(--lz-danger);
  }

  .hero {
    position: relative;
    border-radius: var(--lz-radius);
    overflow: hidden;
    min-height: 240px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    background: var(--lz-surface);
  }

  .hero-img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    filter: brightness(0.6);
  }

  .hero-overlay {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      to bottom,
      rgba(0, 0, 0, 0.3) 0%,
      rgba(0, 0, 0, 0.75) 100%
    );
  }

  .back-btn {
    position: relative;
    z-index: 2;
    align-self: flex-start;
    margin: 12px;
    padding: 6px 12px;
    background: var(--lz-glass);
    backdrop-filter: blur(8px);
    border: 1px solid var(--lz-surface-2);
    border-radius: var(--lz-radius);
    color: var(--lz-text);
    font-size: 13px;
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .back-btn:hover {
    background: var(--lz-surface-2);
    border-color: var(--lz-cyan);
    color: var(--lz-cyan);
  }

  .hero-meta {
    position: relative;
    z-index: 2;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .hero-meta h1 {
    font-size: 26px;
    margin: 0;
    color: var(--lz-text);
    font-weight: 700;
    text-shadow: 0 2px 4px rgba(0, 0, 0, 0.6);
  }

  .meta-row {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
    font-size: 13px;
    color: var(--lz-text-dim);
  }

  .rating {
    color: #ffd700;
    font-weight: 600;
  }

  .genre-row {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin-top: 4px;
  }

  .genre-chip {
    font-size: 11px;
    padding: 3px 8px;
    border-radius: 12px;
    background: var(--lz-surface-2);
    color: var(--lz-cyan);
    border: 1px solid rgba(0, 223, 216, 0.3);
  }

  /* Action Bar */
  .action-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px 16px;
    background: var(--lz-surface);
    border: 1px solid var(--lz-surface-2);
    border-radius: var(--lz-radius);
  }

  .action-left {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .play-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 24px;
    font-size: 15px;
    font-weight: 700;
    border-radius: var(--lz-radius);
    border: none;
    background: linear-gradient(135deg, var(--lz-primary), var(--lz-cyan));
    color: var(--lz-bg);
    cursor: pointer;
    box-shadow: 0 4px 14px rgba(0, 223, 216, 0.3);
    transition: transform 0.15s ease, box-shadow 0.15s ease;
  }

  .play-btn:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 6px 20px rgba(0, 223, 216, 0.45);
  }

  .play-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .play-icon {
    font-size: 14px;
  }

  .manage-dropdown {
    position: relative;
  }

  .manage-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 9px 14px;
    font-size: 13px;
    font-weight: 600;
    border-radius: var(--lz-radius);
    background: var(--lz-surface-2);
    border: 1px solid var(--lz-surface-2);
    color: var(--lz-text);
    cursor: pointer;
    transition: border-color 0.15s ease, background 0.15s ease;
  }

  .manage-btn:hover {
    border-color: var(--lz-cyan);
    color: var(--lz-cyan);
  }

  .arrow {
    font-size: 10px;
    opacity: 0.7;
  }

  .menu-popover {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    z-index: 50;
    min-width: 210px;
    background: var(--lz-surface);
    border: 1px solid var(--lz-surface-2);
    border-radius: var(--lz-radius);
    padding: 6px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
  }

  .pop-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    border-radius: 4px;
    background: transparent;
    border: none;
    color: var(--lz-text);
    font-size: 12.5px;
    text-align: left;
    cursor: pointer;
    transition: background 0.12s ease;
  }

  .pop-item:hover {
    background: var(--lz-surface-2);
    color: var(--lz-cyan);
  }

  .pop-item.danger:hover {
    color: var(--lz-danger);
  }

  .pop-divider {
    border: none;
    border-top: 1px solid var(--lz-surface-2);
    margin: 4px 0;
  }

  .heart-btn {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: var(--lz-surface-2);
    border: 1px solid var(--lz-surface-2);
    color: var(--lz-text-dim);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: color 0.15s ease, border-color 0.15s ease;
  }

  .heart-btn:hover {
    border-color: var(--lz-danger);
    color: var(--lz-danger);
  }

  .heart-btn.filled {
    color: var(--lz-danger);
    border-color: rgba(255, 68, 102, 0.3);
  }

  /* Stats Grid */
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: var(--lz-gap);
  }

  .stat-card {
    background: var(--lz-surface);
    border: 1px solid var(--lz-surface-2);
    border-radius: var(--lz-radius);
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .stat-title {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--lz-text-dim);
    font-weight: 600;
  }

  .stat-num {
    font-size: 18px;
    font-weight: 700;
    color: var(--lz-text);
  }

  .stat-num.glow {
    color: var(--lz-cyan);
  }

  /* Section Cards */
  .section-card {
    background: var(--lz-surface);
    border: 1px solid var(--lz-surface-2);
    border-radius: var(--lz-radius);
    padding: 18px 20px;
  }

  .section-card h2 {
    font-size: 16px;
    margin: 0 0 12px 0;
    color: var(--lz-text);
  }

  .description-text {
    margin: 0;
    color: var(--lz-text-dim);
    font-size: 13.5px;
    line-height: 1.6;
    white-space: pre-line;
  }
  /* Details List */
  .details-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .detail-row {
    display: flex;
    align-items: baseline;
    gap: 12px;
    font-size: 13px;
  }

  .detail-label {
    width: 140px;
    color: var(--lz-text-dim);
    flex-shrink: 0;
  }

  .detail-val {
    color: var(--lz-text);
  }

  .detail-path {
    background: var(--lz-surface-2);
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 12px;
    color: var(--lz-cyan);
    word-break: break-all;
  }

  /* Confirmation Box & Toast */
  .confirm-box {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: rgba(255, 68, 102, 0.1);
    border: 1px solid var(--lz-danger);
    border-radius: var(--lz-radius);
    gap: 12px;
  }

  .confirm-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 13px;
    color: var(--lz-text);
  }

  .confirm-btns {
    display: flex;
    gap: 8px;
  }

  .btn {
    padding: 6px 14px;
    border-radius: var(--lz-radius);
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
    border: none;
  }

  .btn.danger {
    background: var(--lz-danger);
    color: #fff;
  }

  .btn.secondary {
    background: var(--lz-surface-2);
    color: var(--lz-text);
    border: 1px solid var(--lz-surface-2);
  }

  .toast-box {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 14px;
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
    cursor: pointer;
    font-size: 14px;
  }
</style>
