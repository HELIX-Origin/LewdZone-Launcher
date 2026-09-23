<svelte:options runes={true} />

<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
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
    status: "queued" | "resolving" | "dispatching" | "dispatched" | "failed";
    message: string | null;
    manager: string | null;
    created_at: number;
    updated_at: number;
  }

  interface HostSource {
    host: string;
    label: string;
    preferred: boolean;
  }

  const slug = $derived((page.url.pathname.match(/\/store\/(.+)/) ?? [])[1] ?? "");
  let status: LoadState = $state("loading");
  let error = $state("");
  let game: GameData | null = $state(null);

  let version = $state("latest");
  let platform = $state("PC");
  let tab = $state("official");
  let sources: HostSource[] = $state([]);
  let source = $state("");
  let downloading = $state(false);
  let feedback = $state("");

  const platformLabel: Record<string, string> = {
    pc: "PC",
    android: "Android",
    mac: "Mac",
    linux: "Linux",
  };

  async function load() {
    status = "loading";
    try {
      const data = await invoke<GameData>("game_page", { slug });
      game = data;
      if (data.versions.length > 0) {
        version = data.versions.find((v) => v.is_latest)?.label ?? data.versions[0].label;
      }
      await refreshSources();
      status = "ready";
    } catch (err) {
      status = "error";
      error = String(err);
    }
  }

  async function refreshSources() {
    if (!game) return;
    try {
      const list = await invoke<HostSource[]>("game_sources", {
        slug: game.slug,
        version: version === "latest" ? "latest" : version,
        platform: platform === "PC" ? "PC" : platform,
        tab,
      });
      sources = list;
      // Preferred source (when configured & available) is the default; else
      // the first listed source.
      const fallback = list.find((s) => s.host === source)?.host;
      const chosen = list.find((s) => s.preferred)?.host ?? fallback ?? list[0]?.host ?? "";
      source = chosen;
    } catch (err) {
      sources = [];
      source = "";
      feedback = `Sources unavailable: ${String(err)}`;
    }
  }

  function onSelectionChange() {
    refreshSources();
  }

  onMount(() => load());

  async function download() {
    if (!game) return;
    downloading = true;
    feedback = "";
    try {
      const job = await invoke<Job>("game_download", {
        slug: game.slug,
        version: version === "latest" ? "latest" : version,
        platform: platform === "PC" ? "PC" : platform,
        tab,
        source: source || null,
      });
      feedback = `Queued download #${job.id}. Watch the Downloads page for progress.`;
    } catch (err) {
      feedback = `Download failed: ${String(err)}`;
    } finally {
      downloading = false;
    }
  }

  function back() {
    goto("/store");
  }
</script>

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
      <button class="back" onclick={back}
        ><svg class="back-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M15 5l-7 7 7 7"/></svg>Back to Store</button
      >
      <div class="hero-meta">
        <h1>{game.title}</h1>
        <div class="meta-row">
          {#if game.developer}<span>by {game.developer}</span>{/if}
          {#if game.current_version}<span>v{game.current_version}</span>{/if}
          {#if game.engine}<span>{game.engine}</span>{/if}
          {#if game.size_label}<span>{game.size_label}</span>{/if}
          {#if game.censorship}<span>{game.censorship}</span>{/if}
        </div>
        <div class="genre-row">
          {#each game.genres as genre (genre)}
            <a class="genre-chip" href={`/store/${genre}`}>{genre}</a>
          {/each}
        </div>
      </div>
    </div>

    {#if game.description}
      <p class="desc">{game.description}</p>
    {/if}

    <div class="dl-panel">
      <h2>Download</h2>
      <div class="dl-controls">
        <label>
          Version
          <select bind:value={version} onchange={onSelectionChange}>
            {#if !game.versions.some((v) => v.label === "latest")}
              <option value="latest">(latest)</option>
            {/if}
            {#each game.versions as v (v.label)}
              <option value={v.label}>{v.label}{v.is_latest ? " (latest)" : ""}</option>
            {/each}
          </select>
        </label>
        <label>
          Platform
          <select bind:value={platform} onchange={onSelectionChange}>
            {#each ["PC", "Android", "Mac", "Linux"] as p (p)}
              <option value={p}>{p}</option>
            {/each}
          </select>
        </label>
        <label>
          Tab
          <select bind:value={tab} onchange={onSelectionChange}>
            <option value="official">Official</option>
            <option value="community">Community</option>
          </select>
        </label>
        <label>
          Source
          <select bind:value={source} disabled={sources.length === 0}>
            {#each sources as s (s.host)}
              <option value={s.host}>
                {s.label}{s.preferred ? " (preferred)" : ""}
              </option>
            {/each}
          </select>
        </label>
        <button
          class="dl-btn"
          onclick={download}
          disabled={downloading || sources.length === 0}
          aria-label={`Download ${game.title}`}
        >
          {downloading ? "Queueing…" : "⬇ Download"}
        </button>
      </div>
      {#if feedback}
        <p class="feedback" role="status">{feedback}</p>
      {/if}
    </div>

    {#if game.download_entries.length > 0}
      <div class="entries">
        <h2>All download links ({game.download_entries.length})</h2>
        <div class="entry-table" role="list">
          {#each game.download_entries as entry (entry.go_link)}
            <div class="entry-row" role="listitem">
              <span class="entry-label">{entry.label}</span>
              <span class="entry-host">{entry.host}</span>
              <span class="entry-plat">
                {entry.platform ? (platformLabel[entry.platform] ?? entry.platform) : ""}
              </span>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .detail {
    padding: var(--lz-gap);
    max-width: 980px;
    margin: 0 auto;
  }

  .hero {
    position: relative;
    border-radius: var(--lz-radius);
    overflow: hidden;
    min-height: 240px;
    background: var(--lz-surface);
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
  }

  .hero-img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    opacity: 0.55;
  }

  .hero-overlay {
    position: absolute;
    inset: 0;
    background: linear-gradient(180deg, transparent 0%, rgba(10, 17, 24, 0.95) 100%);
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
    text-decoration: none;
  }

  .desc {
    color: var(--lz-text-dim);
    line-height: 1.5;
  }

  .dl-panel {
    background: var(--lz-surface);
    border: 1px solid var(--lz-surface-2);
    border-radius: var(--lz-radius);
    padding: var(--lz-gap);
    margin: var(--lz-gap) 0;
  }

  .dl-panel h2,
  .entries h2 {
    font-size: 15px;
    margin: 0 0 10px;
  }

  .dl-controls {
    display: flex;
    flex-wrap: wrap;
    gap: var(--lz-gap);
    align-items: flex-end;
  }

  .dl-controls label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
    color: var(--lz-text-dim);
  }

  select {
    background: var(--lz-surface-2);
    color: var(--lz-text);
    border: 1px solid var(--lz-surface-2);
    border-radius: var(--lz-radius);
    font: inherit;
    padding: 6px 8px;
  }

  .dl-btn {
    appearance: none;
    border: none;
    background: linear-gradient(135deg, var(--lz-primary), var(--lz-accent));
    color: #fff;
    font-weight: 700;
    font: inherit;
    padding: 8px 18px;
    border-radius: var(--lz-radius);
    cursor: pointer;
  }

  .dl-btn:disabled {
    opacity: 0.6;
    cursor: default;
  }

  .feedback {
    margin-top: 10px;
    font-size: 13px;
    color: var(--lz-text);
  }

  .entry-table {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .entry-row {
    display: grid;
    grid-template-columns: 1fr auto auto;
    gap: var(--lz-gap);
    padding: 6px 10px;
    background: var(--lz-surface);
    border: 1px solid var(--lz-surface-2);
    border-radius: var(--lz-radius);
    font-size: 13px;
  }

  .entry-host {
    color: var(--lz-text-dim);
  }

  .entry-plat {
    color: var(--lz-cyan);
  }

  .note {
    color: var(--lz-text-dim);
  }

  .note.error {
    color: var(--lz-danger);
  }
</style>