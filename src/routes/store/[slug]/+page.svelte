<svelte:options runes={true} />

<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";

  type LoadState = "loading" | "ready" | "error";

  interface DownloadEntry {
    label: string;
    variant: string | null;
    host: string;
    platform: string | null;
    go_link: string;
  }

  interface Version {
    label: string;
    is_latest: boolean;
    official: DownloadEntry[];
    community: DownloadEntry[];
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
    versions: Version[];
    download_entries: DownloadEntry[];
  }

  interface Job {
    id: number;
    slug: string;
    version: string;
    platform: string;
    tab: string;
    source: string | null;
    status: "queued" | "resolving" | "dispatching" | "downloading" | "dispatched" | "failed";
    message: string | null;
    bytes_done: number;
    bytes_total: number;
    created_at: number;
    updated_at: number;
  }

  interface HostSource {
    host: string;
    label: string;
    preferred: boolean;
  }

  interface GroupedDownloads {
    platform: string;
    variant: string;
    entries: DownloadEntry[];
  }

  // Known dead, malicious, or non-functional hosts on lewdzone.com
  // Any host parsed from the site NOT in this set is considered supported.
  const UNSUPPORTED_HOSTS = new Set([
    "gofile",
    "anonfiles",
    "bayfiles",
    "zippyshare",
    "uptobox",
    "clicknupload",
  ]);

  function isSupportedHost(h: string | null | undefined): boolean {
    if (!h) return false;
    const clean = h.trim().toLowerCase();
    return clean.length > 0 && !UNSUPPORTED_HOSTS.has(clean);
  }

  const slug = $derived(
    (page.params?.slug ?? (page.url.pathname.match(/\/store\/([^/?#]+)/) ?? [])[1] ?? "")
      .replace(/\/+$/, "")
  );

  let status: LoadState = $state("loading");
  let error = $state("");
  let game: GameData | null = $state(null);

  let version = $state("latest");
  let tab = $state("official");
  let activeDownloadEntry: DownloadEntry | null = $state(null);
  let downloading = $state(false);
  let feedback = $state("");

  // Preview Carousel State
  let activeImageIndex = $state(0);
  let isLightboxOpen = $state(false);

  const platformLabel: Record<string, string> = {
    pc: "Windows PC",
    windows: "Windows PC",
    android: "Android",
    mac: "Mac OS",
    linux: "Linux",
  };

  let activeSlug = "";
  let inFlightSlug: string | null = null;
  let unlistenResolved: UnlistenFn | undefined;

  onMount(async () => {
    if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
      try {
        unlistenResolved = await listen<{ slug: string; url: string }>(
          "download-url-resolved",
          async (event) => {
            if (event.payload.slug === game?.slug) {
              feedback = `Captured verified link: ${event.payload.url}. Starting download…`;
              if (activeDownloadEntry) {
                await queueDownload(activeDownloadEntry);
              }
            }
          }
        );
      } catch (e) {
        console.warn("Could not attach download-url-resolved listener:", e);
      }
    }
  });

  onDestroy(() => {
    if (unlistenResolved) {
      unlistenResolved();
    }
  });

  async function load(targetSlug?: string) {
    const s = targetSlug !== undefined ? targetSlug : slug;
    if (activeSlug === s && status === "ready" && game) {
      return;
    }
    if (inFlightSlug === s) {
      return;
    }
    inFlightSlug = s;
    activeSlug = s;
    status = "loading";
    error = "";
    try {
      const data = await invoke<GameData>("game_page", { slug: s });
      if (activeSlug !== s) return;
      game = {
        ...data,
        screenshots: data.screenshots ?? [],
        genres: data.genres ?? [],
        versions: data.versions ?? [],
        download_entries: data.download_entries ?? [],
      };
      activeImageIndex = 0;
      if (data.versions && data.versions.length > 0) {
        const found =
          data.versions.find(
            (v) =>
              v.is_latest &&
              ((v.official && v.official.some((e) => isSupportedHost(e.host))) ||
                (v.community && v.community.some((e) => isSupportedHost(e.host))))
          ) ??
          data.versions.find(
            (v) =>
              (v.official && v.official.some((e) => isSupportedHost(e.host))) ||
              (v.community && v.community.some((e) => isSupportedHost(e.host)))
          ) ??
          data.versions[0];
        version = found.label;
      } else {
        version = "latest";
      }
      status = "ready";
    } catch (err) {
      if (activeSlug !== s) return;
      status = "error";
      error = String(err);
    } finally {
      if (inFlightSlug === s) {
        inFlightSlug = null;
      }
    }
  }

  $effect(() => {
    const s = slug;
    load(s);
  });

  async function queueDownload(entry: DownloadEntry) {
    if (!game) return;
    downloading = true;
    feedback = "";
    try {
      const job = await invoke<Job>("game_download", {
        slug: game.slug,
        version: currentVersionObj?.label ?? (version === "latest" ? "latest" : version),
        platform: entry.platform ? entry.platform.toUpperCase() : "PC",
        tab,
        source: entry.host || null,
      });
      feedback = `Queued download #${job.id} (${formatHostName(entry.host)}). Watch the Downloads page for progress.`;
    } catch (err) {
      feedback = `Download failed: ${String(err)}`;
    } finally {
      downloading = false;
    }
  }

  async function startDirectEntryDownload(entry: DownloadEntry) {
    if (!game) return;
    activeDownloadEntry = entry;
    feedback = `Opening secure resolver for ${entry.label} (${formatHostName(entry.host)})…`;
    try {
      await invoke("open_resolver_window", {
        slug: game.slug,
        url: entry.go_link,
        host: entry.host,
        title: `${game.title} - ${entry.label}`,
      });
    } catch (err) {
      feedback = `Resolver failed: ${String(err)}`;
    }
  }

  const HOST_ICONS: Record<string, string> = {
    mega: `<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="11" fill="#D9272E"/><path d="M7 16V8l5 4.5L17 8v8h-2v-5l-3 2.7-3-2.7V16H7z" fill="#FFF"/></svg>`,
    google: `<svg viewBox="0 0 24 24"><path d="M8.5 3.5l-6 10.5 3.5 6 6-10.5z" fill="#0066DA"/><path d="M15.5 3.5H8.5l6 10.5h7z" fill="#00AC47"/><path d="M21.5 14H9.5l-3.5 6h12z" fill="#EA4335"/><path d="M6 14l3.5-6-3.5-6L0 12z" fill="#FFBA00"/></svg>`,
    dropbox: `<svg viewBox="0 0 24 24"><path d="M6 3.5L0 7.5l6 4 6-4-6-4zm12 0l-6 4 6 4 6-4-6-4zM0 15.5l6 4 6-4-6-4-6 4zm24 0l-6-4-6 4 6 4 6-4zM6 21l6-4 6 4-6 4-6-4z" fill="#0061FF"/></svg>`,
    mediafire: `<svg viewBox="0 0 24 24"><path d="M17.5 10.5c-.5-3.5-3.5-6-7-6-2.5 0-4.8 1.4-5.9 3.5C2 8.5 0 11 0 14c0 3.3 2.7 6 6 6h11.5c3.6 0 6.5-2.9 6.5-6.5 0-3.3-2.5-6-5.8-6.5z" fill="#1299F3"/><path d="M12 7c-1.5 2-2.5 4-2.5 6 0 2 1.5 3.5 3.5 3.5s3.5-1.5 3.5-3.5c0-2-1.5-4-3-6-.3-.4-.8-.4-1.5 0z" fill="#FFF"/></svg>`,
    pixeldrain: `<svg viewBox="0 0 24 24"><path d="M12 2C8 7 5 11 5 15a7 7 0 0 0 14 0c0-4-3-8-7-13z" fill="#FF6B4A"/><circle cx="10" cy="14" r="2.5" fill="#FFF"/></svg>`,
    fileknot: `<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="#7928CA"/><path d="M8 8a4 4 0 0 1 8 0v8a4 4 0 0 1-8 0V8z" fill="none" stroke="#FFF" stroke-width="2.2"/><circle cx="12" cy="12" r="2.2" fill="#00DFD8"/></svg>`,
    transfaze: `<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="#00C08B"/><path d="M7 10l5-4 5 4M17 14l-5 4-5-4" stroke="#FFF" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" fill="none"/></svg>`,
    workupload: `<svg viewBox="0 0 24 24"><rect x="3" y="4" width="18" height="16" rx="4" fill="#2ECC71"/><path d="M12 8v8M8 12l4-4 4 4" stroke="#FFF" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
    uploadhaven: `<svg viewBox="0 0 24 24"><path d="M12 2L3 6v6c0 5.5 3.8 10.7 9 12 5.2-1.3 9-6.5 9-12V6l-9-4z" fill="#0E76A8"/><path d="M12 7l4 4h-3v5h-2v-5H8l4-4z" fill="#FFF"/></svg>`,
    "1fichier": `<svg viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="4" fill="#FF6600"/><path d="M10 8l3-2v12h-3" fill="none" stroke="#FFF" stroke-width="2.8" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
    rapidgator: `<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="#E65100"/><path d="M9 7h4a3 3 0 0 1 0 6H9zm0 6l5 5" stroke="#FFF" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" fill="none"/></svg>`,
    qiwi: `<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="#FF8C00"/><circle cx="12" cy="12" r="5.5" fill="#FFF"/><circle cx="13.5" cy="10.5" r="1.8" fill="#FF8C00"/></svg>`,
    bowfile: `<svg viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="4" fill="#3B82F6"/><path d="M7 17L17 7M7 7h6M17 17V11" stroke="#FFF" stroke-width="2" stroke-linecap="round"/></svg>`,
    hexload: `<svg viewBox="0 0 24 24"><polygon points="12 2 21 7 21 17 12 22 3 17 3 7" fill="#8B5CF6"/><polyline points="8 12 12 16 16 8" stroke="#FFF" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none"/></svg>`,
    mixdrop: `<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="#EC4899"/><path d="M12 6v8M9 11l3 3 3-3" stroke="#FFF" stroke-width="2" stroke-linecap="round" fill="none"/></svg>`,
    racaty: `<svg viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="4" fill="#10B981"/><path d="M7 12h10M12 7v10" stroke="#FFF" stroke-width="2" stroke-linecap="round"/></svg>`,
    terminal: `<svg viewBox="0 0 24 24"><rect x="2" y="4" width="20" height="16" rx="3" fill="#1E293B"/><path d="M6 9l3 3-3 3M11 15h6" stroke="#10B981" stroke-width="2" stroke-linecap="round"/></svg>`,
  };

  function getHostIcon(host: string): string {
    const key = host.toLowerCase();
    return (
      HOST_ICONS[key] ??
      `<svg viewBox="0 0 24 24"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none"/></svg>`
    );
  }

  function formatHostName(host: string): string {
    const map: Record<string, string> = {
      fileknot: "Fileknot",
      transfaze: "Transfaze",
      mega: "Mega",
      google: "Google Drive",
      uploadhaven: "UploadHaven",
      workupload: "Workupload",
      mediafire: "Mediafire",
      dropbox: "Dropbox",
      pixeldrain: "Pixeldrain",
      mixdrop: "MixDrop",
      racaty: "Racaty",
      terminal: "Terminal",
      "1fichier": "1Fichier",
      rapidgator: "Rapidgator",
      qiwi: "Qiwi",
      bowfile: "Bowfile",
      hexload: "Hexload",
    };
    return map[host.toLowerCase()] ?? host.charAt(0).toUpperCase() + host.slice(1);
  }

  // Available versions that have active download entries on the scraped page
  const availableVersions = $derived.by<Version[]>(() => {
    if (!game) return [];
    if (game.versions && game.versions.length > 0) {
      const withLinks = game.versions.filter(
        (v) =>
          (v.official && v.official.some((e) => isSupportedHost(e.host))) ||
          (v.community && v.community.some((e) => isSupportedHost(e.host)))
      );
      if (withLinks.length > 0) return withLinks;
      return game.versions;
    }
    if (game.download_entries && game.download_entries.length > 0) {
      return [
        {
          label: game.current_version || "latest",
          is_latest: true,
          official: game.download_entries,
          community: [],
        },
      ];
    }
    return [];
  });

  // Active version entries grouped by platform and variant (LewdZone layout)
  const currentVersionObj = $derived.by<Version | undefined>(() => {
    if (availableVersions.length === 0) return undefined;
    return availableVersions.find((v: Version) => v.label === version) ?? availableVersions[0];
  });

  const activeDownloadGroups = $derived.by<GroupedDownloads[]>(() => {
    if (!currentVersionObj) return [];
    const rawList: DownloadEntry[] =
      tab === "community" ? currentVersionObj.community : currentVersionObj.official;
    // Omit unsupported hosts (anything not in UNSUPPORTED_HOSTS is supported)
    const supportedList = (rawList ?? []).filter(
      (e: DownloadEntry) => e && e.go_link && isSupportedHost(e.host)
    );

    const groupMap = new Map<string, DownloadEntry[]>();
    for (const entry of supportedList) {
      const plat = entry.platform ? (platformLabel[entry.platform.toLowerCase()] ?? entry.platform) : "General";
      const variant = entry.variant || "Standard";
      const key = `${plat}:::${variant}`;
      if (!groupMap.has(key)) {
        groupMap.set(key, []);
      }
      groupMap.get(key)!.push(entry);
    }

    const groups: GroupedDownloads[] = [];
    for (const [key, entries] of groupMap.entries()) {
      const [p, v] = key.split(":::");
      groups.push({ platform: p, variant: v, entries });
    }
    return groups;
  });

  // Carousel controls
  function nextImage() {
    if (!game || game.screenshots.length === 0) return;
    activeImageIndex = (activeImageIndex + 1) % game.screenshots.length;
  }

  function prevImage() {
    if (!game || game.screenshots.length === 0) return;
    activeImageIndex = (activeImageIndex - 1 + game.screenshots.length) % game.screenshots.length;
  }

  function selectImage(index: number) {
    activeImageIndex = index;
  }

  function toggleLightbox() {
    isLightboxOpen = !isLightboxOpen;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "ArrowRight") nextImage();
    if (e.key === "ArrowLeft") prevImage();
    if (e.key === "Escape" && isLightboxOpen) isLightboxOpen = false;
  }

  function back() {
    goto("/store");
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="detail">
  {#if status === "loading"}
    <p class="note">Loading game…</p>
  {:else if status === "error"}
    <p class="note error">{error}</p>
  {:else if game}
    <div class="hero">
      {#if game.screenshots.length > 0}
        <img class="hero-img" src={game.screenshots[0]} alt="" aria-hidden="true" />
      {/if}
      <div class="hero-overlay"></div>
      <button class="back" onclick={back}>
        <svg class="back-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M15 5l-7 7 7 7"/></svg>
        Back to Store
      </button>
      <div class="hero-meta">
        <h1>{game.title}</h1>
        <div class="meta-row">
          {#if game.developer}<span>by {game.developer}</span>{/if}
          {#if game.current_version}<span>v{game.current_version}</span>{/if}
          {#if game.engine}<span>{game.engine}</span>{/if}
          {#if game.size_label}<span>{game.size_label}</span>{/if}
          {#if game.censorship}<span>{game.censorship}</span>{/if}
          {#if typeof game.rating === "number" && game.rating > 0}<span>★ {game.rating.toFixed(1)}</span>{/if}
        </div>
        <div class="genre-row">
          {#each game.genres as genre (genre)}
            <span class="genre-chip">{genre}</span>
          {/each}
        </div>
      </div>
    </div>

    {#if game.description}
      <p class="desc">{game.description}</p>
    {/if}

    <!-- Preview Carousel Section -->
    {#if game.screenshots.length > 0}
      <section class="carousel-section" aria-label="Game Preview Images">
        <div class="carousel-header">
          <h2>
            <svg class="section-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
              <circle cx="8.5" cy="8.5" r="1.5"/>
              <polyline points="21 15 16 10 5 21"/>
            </svg>
            Attached Preview Images
          </h2>
          <span class="carousel-counter">{activeImageIndex + 1} / {game.screenshots.length}</span>
        </div>

        <div class="carousel-stage">
          <button class="nav-arrow left" onclick={prevImage} aria-label="Previous image">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="15 18 9 12 15 6"></polyline>
            </svg>
          </button>

          <button class="stage-img-btn" onclick={toggleLightbox} aria-label="Click to enlarge image">
            <img
              class="stage-img"
              src={game.screenshots[activeImageIndex]}
              alt={`${game.title} preview screenshot ${activeImageIndex + 1}`}
            />
            <div class="stage-overlay">
              <span class="zoom-badge">🔍 Click for Fullscreen</span>
            </div>
          </button>

          <button class="nav-arrow right" onclick={nextImage} aria-label="Next image">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="9 18 15 12 9 6"></polyline>
            </svg>
          </button>
        </div>

        <!-- Thumbnail Strip -->
        <div class="thumbnail-strip" role="tablist" aria-label="Thumbnails">
          {#each game.screenshots as thumbUrl, idx (thumbUrl)}
            <button
              class="thumb-btn"
              class:active={idx === activeImageIndex}
              onclick={() => selectImage(idx)}
              aria-label={`View image ${idx + 1}`}
              role="tab"
              aria-selected={idx === activeImageIndex}
            >
              <img src={thumbUrl} alt="" loading="lazy" />
            </button>
          {/each}
        </div>
      </section>
    {/if}

    <!-- Lightbox Modal -->
    {#if isLightboxOpen && game.screenshots.length > 0}
      <div class="lightbox" role="dialog" aria-modal="true">
        <button class="lightbox-close" onclick={toggleLightbox} aria-label="Close fullscreen view">✕</button>
        <button class="lightbox-nav left" onclick={prevImage} aria-label="Previous">❮</button>
        <img
          class="lightbox-img"
          src={game.screenshots[activeImageIndex]}
          alt={`${game.title} full view`}
        />
        <button class="lightbox-nav right" onclick={nextImage} aria-label="Next">❯</button>
        <div class="lightbox-caption">{activeImageIndex + 1} of {game.screenshots.length}</div>
      </div>
    {/if}

    <!-- LewdZone-Style Downloads Section -->
    <div class="dl-panel">
      <div class="dl-header-row">
        <h2>Download Game</h2>
        {#if availableVersions.length > 1}
          <div class="version-select-box">
            <label for="version-select">Choose Version:</label>
            <select id="version-select" bind:value={version}>
              {#each availableVersions as v (v.label)}
                <option value={v.label}>{v.label}{v.is_latest ? " (Latest)" : ""}</option>
              {/each}
            </select>
          </div>
        {:else if availableVersions.length === 1}
          <div class="version-badge">
            <span class="version-label">Version:</span>
            <span class="version-value">{availableVersions[0].label}</span>
          </div>
        {/if}
      </div>

      <!-- Official vs Community Tabs -->
      <div class="tab-pills" role="tablist">
        <button
          class="tab-pill"
          class:active={tab === "official"}
          onclick={() => { tab = "official"; }}
          role="tab"
          aria-selected={tab === "official"}
        >
          Official Links ({currentVersionObj?.official?.filter((e) => isSupportedHost(e.host))?.length ?? 0})
        </button>
        {#if (currentVersionObj?.community?.filter((e) => isSupportedHost(e.host))?.length ?? 0) > 0}
          <button
            class="tab-pill"
            class:active={tab === "community"}
            onclick={() => { tab = "community"; }}
            role="tab"
            aria-selected={tab === "community"}
          >
            Community Links ({currentVersionObj?.community?.filter((e) => isSupportedHost(e.host))?.length ?? 0})
          </button>
        {/if}
      </div>

      {#if feedback}
        <p class="feedback" role="status">{feedback}</p>
      {/if}

      <!-- LewdZone Grouped Per-Source Download Buttons -->
      <div class="lz-downloads-container">
        <h3>Available Download Sources ({tab === "official" ? "Official" : "Community"})</h3>
        {#if activeDownloadGroups.length === 0}
          <p class="empty-sources-notice">No supported download links available for this version and tab.</p>
        {:else}
          <div class="lz-groups-list">
            {#each activeDownloadGroups as group (group.platform + group.variant)}
              <div class="lz-group-card">
                <div class="lz-group-header">
                  <span class="group-platform-tag">{group.platform}</span>
                  {#if group.variant && group.variant !== "Standard"}
                    <span class="group-variant-tag">{group.variant}</span>
                  {/if}
                </div>
                <div class="lz-source-buttons">
                  {#each group.entries as entry (entry.go_link + entry.host)}
                    <button
                      class="lz-source-btn"
                      onclick={() => startDirectEntryDownload(entry)}
                      title={`Download via ${formatHostName(entry.host)}`}
                      aria-label={`Download via ${formatHostName(entry.host)}`}
                    >
                      <span class="provider-icon">
                        {@html getHostIcon(entry.host)}
                      </span>
                      <span class="host-text">{formatHostName(entry.host)}</span>
                    </button>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .detail {
    padding: var(--lz-gap);
    max-width: 960px;
    margin: 0 auto;
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
    filter: brightness(0.65);
  }

  .hero-overlay {
    position: absolute;
    inset: 0;
    background: linear-gradient(to top, rgba(15, 17, 23, 0.95), transparent 70%);
  }

  .back {
    position: relative;
    appearance: none;
    border: none;
    background: var(--lz-glass);
    color: var(--lz-text);
    font: inherit;
    font-size: 13px;
    padding: 6px 12px;
    border-radius: var(--lz-radius);
    cursor: pointer;
    align-self: flex-start;
    margin: 10px;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .back-icon {
    width: 16px;
    height: 16px;
  }

  .hero-meta {
    position: relative;
    padding: 12px var(--lz-gap) var(--lz-gap);
    z-index: 1;
  }

  .hero-meta h1 {
    margin: 0 0 6px;
    font-size: 22px;
  }

  .meta-row {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    color: var(--lz-text-dim);
    font-size: 13px;
  }

  .genre-row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 8px;
  }

  .genre-chip {
    color: var(--lz-cyan);
    background: var(--lz-surface-2);
    border-radius: 10px;
    padding: 2px 10px;
    font-size: 12px;
  }

  .desc {
    color: var(--lz-text-dim);
    line-height: 1.5;
    margin: var(--lz-gap) 0;
  }

  /* Carousel Styles */
  .carousel-section {
    background: var(--lz-surface);
    border: 1px solid var(--lz-surface-2);
    border-radius: var(--lz-radius);
    padding: var(--lz-gap);
    margin: var(--lz-gap) 0;
  }

  .carousel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }

  .carousel-header h2 {
    font-size: 16px;
    margin: 0;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .section-icon {
    width: 18px;
    height: 18px;
    color: var(--lz-cyan);
  }

  .carousel-counter {
    font-size: 12px;
    color: var(--lz-text-dim);
    background: var(--lz-surface-2);
    padding: 3px 8px;
    border-radius: 12px;
  }

  .carousel-stage {
    position: relative;
    width: 100%;
    height: 380px;
    background: #000;
    border-radius: var(--lz-radius);
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .stage-img-btn {
    appearance: none;
    border: none;
    background: transparent;
    padding: 0;
    margin: 0;
    width: 100%;
    height: 100%;
    cursor: zoom-in;
    position: relative;
  }

  .stage-img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    transition: transform 0.2s ease;
  }

  .stage-overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.2);
    opacity: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: opacity 0.2s ease;
  }

  .stage-img-btn:hover .stage-overlay {
    opacity: 1;
  }

  .zoom-badge {
    background: rgba(0, 0, 0, 0.7);
    color: #fff;
    padding: 6px 14px;
    border-radius: 20px;
    font-size: 13px;
    border: 1px solid rgba(255, 255, 255, 0.2);
  }

  .nav-arrow {
    position: absolute;
    top: 50%;
    transform: translateY(-50%);
    width: 44px;
    height: 44px;
    border-radius: 50%;
    background: rgba(15, 17, 23, 0.75);
    border: 1px solid rgba(255, 255, 255, 0.15);
    color: #fff;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2;
    transition: background 0.15s ease, transform 0.1s ease;
  }

  .nav-arrow:hover {
    background: rgba(0, 238, 255, 0.85);
    color: #0f1117;
  }

  .nav-arrow.left { left: 12px; }
  .nav-arrow.right { right: 12px; }

  .nav-arrow svg {
    width: 22px;
    height: 22px;
  }

  .thumbnail-strip {
    display: flex;
    gap: 8px;
    margin-top: 12px;
    overflow-x: auto;
    padding-bottom: 4px;
  }

  .thumb-btn {
    appearance: none;
    border: 2px solid transparent;
    border-radius: 6px;
    padding: 0;
    background: #000;
    cursor: pointer;
    flex-shrink: 0;
    width: 80px;
    height: 52px;
    overflow: hidden;
    opacity: 0.6;
    transition: opacity 0.15s ease, border-color 0.15s ease;
  }

  .thumb-btn:hover {
    opacity: 0.9;
  }

  .thumb-btn.active {
    opacity: 1;
    border-color: var(--lz-cyan);
  }

  .thumb-btn img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  /* Lightbox Modal */
  .lightbox {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.92);
    z-index: 1000;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .lightbox-img {
    max-width: 90vw;
    max-height: 85vh;
    object-fit: contain;
    border-radius: 4px;
  }

  .lightbox-close {
    position: absolute;
    top: 20px;
    right: 24px;
    background: rgba(255, 255, 255, 0.1);
    border: none;
    color: #fff;
    font-size: 24px;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    cursor: pointer;
  }

  .lightbox-nav {
    position: absolute;
    top: 50%;
    transform: translateY(-50%);
    background: rgba(255, 255, 255, 0.1);
    border: none;
    color: #fff;
    font-size: 32px;
    width: 50px;
    height: 70px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .lightbox-nav.left { left: 16px; }
  .lightbox-nav.right { right: 16px; }

  .lightbox-caption {
    position: absolute;
    bottom: 20px;
    color: #a0aec0;
    font-size: 14px;
  }

  /* Download Panel & LewdZone Style */
  .dl-panel {
    background: var(--lz-surface);
    border: 1px solid var(--lz-surface-2);
    border-radius: var(--lz-radius);
    padding: var(--lz-gap);
    margin: var(--lz-gap) 0;
  }

  .dl-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 12px;
    margin-bottom: 14px;
  }

  .dl-header-row h2 {
    font-size: 18px;
    margin: 0;
  }

  .version-select-box {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--lz-text-dim);
  }

  .tab-pills {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
    border-bottom: 1px solid var(--lz-surface-2);
    padding-bottom: 10px;
  }

  .tab-pill {
    appearance: none;
    border: none;
    background: var(--lz-surface-2);
    color: var(--lz-text-dim);
    font: inherit;
    font-size: 13px;
    font-weight: 600;
    padding: 6px 14px;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .tab-pill:hover {
    color: #fff;
    background: rgba(0, 238, 255, 0.15);
  }

  .tab-pill.active {
    background: var(--lz-cyan);
    color: #0f1117;
  }

  .version-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    background: var(--lz-surface-2);
    padding: 4px 10px;
    border-radius: 6px;
  }

  .version-label {
    color: var(--lz-text-dim);
  }

  .version-value {
    color: var(--lz-cyan);
    font-weight: 600;
  }

  select {
    background: var(--lz-surface-2);
    color: var(--lz-text);
    border: 1px solid var(--lz-surface-2);
    border-radius: var(--lz-radius);
    font: inherit;
    padding: 6px 8px;
  }

  .feedback {
    background: rgba(0, 238, 255, 0.1);
    border: 1px solid rgba(0, 238, 255, 0.3);
    color: var(--lz-cyan);
    padding: 8px 12px;
    border-radius: 6px;
    font-size: 13px;
    margin: 12px 0;
  }

  /* LewdZone Grouped Cards */
  .lz-downloads-container h3 {
    font-size: 14px;
    color: var(--lz-text-dim);
    margin: 0 0 10px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .empty-sources-notice {
    color: var(--lz-text-dim);
    font-size: 13px;
    padding: 12px;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 6px;
  }

  .lz-groups-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .lz-group-card {
    background: var(--lz-surface-2);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    padding: 12px 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .lz-group-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .group-platform-tag {
    font-weight: 700;
    font-size: 13px;
    color: #fff;
  }

  .group-variant-tag {
    font-size: 11px;
    color: var(--lz-cyan);
    background: rgba(0, 238, 255, 0.12);
    padding: 2px 6px;
    border-radius: 4px;
    font-weight: 600;
  }

  .lz-source-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .lz-source-btn {
    appearance: none;
    border: 1px solid rgba(0, 238, 255, 0.25);
    background: rgba(0, 238, 255, 0.08);
    color: #fff;
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    transition: all 0.15s ease;
  }

  .lz-source-btn:hover {
    background: var(--lz-cyan);
    color: #0f1117;
    border-color: var(--lz-cyan);
  }

  .provider-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    flex-shrink: 0;
  }

  .provider-icon :global(svg) {
    width: 16px;
    height: 16px;
    display: block;
  }
</style>