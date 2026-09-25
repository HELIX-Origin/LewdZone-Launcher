<svelte:options runes={true} />

<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { cleanDisplayTitle, formatCanonicalTitle, formatDisplayMessage } from "$lib/format";

  type LoadState = "loading" | "ready" | "error";

  type Status =
    | "queued"
    | "resolving"
    | "dispatching"
    | "downloading"
    | "extracting"
    | "dispatched"
    | "completed"
    | "failed";

  interface QueueJob {
    id: number;
    slug: string;
    title: string;
    version: string;
    platform: string;
    tab: string;
    source: string | null;
    status: Status;
    message: string | null;
    bytes_done: number;
    bytes_total: number;
    created_at: number;
    updated_at: number;
  }

  const statusLabel: Record<Status, string> = {
    queued: "Queued",
    resolving: "Resolving",
    dispatching: "Dispatching",
    downloading: "Downloading",
    extracting: "Extracting",
    dispatched: "Dispatched",
    completed: "Completed",
    failed: "Failed",
  };

  let status: LoadState = $state("loading");
  let error = $state("");
  let jobs: QueueJob[] = $state([]);
  let isRefreshing = $state(false);

  function hasActiveJobs(list: QueueJob[]): boolean {
    return list.some(
      (j) =>
        j.status === "queued" ||
        j.status === "resolving" ||
        j.status === "dispatching" ||
        j.status === "downloading" ||
        j.status === "extracting"
    );
  }

  let timer: ReturnType<typeof setInterval> | undefined;

  function ensurePolling(list: QueueJob[]) {
    if (hasActiveJobs(list)) {
      if (!timer) {
        timer = setInterval(async () => {
          await poll();
        }, 1500);
      }
    } else {
      if (timer) {
        clearInterval(timer);
        timer = undefined;
      }
    }
  }

  async function poll() {
    try {
      jobs = await invoke<QueueJob[]>("downloads_list");
      ensurePolling(jobs);
    } catch {
      // Background poll silently fails without disrupting UI
    }
  }

  async function refresh(isManual = false) {
    if (isManual) isRefreshing = true;
    try {
      jobs = await invoke<QueueJob[]>("downloads_list");
      status = "ready";
      ensurePolling(jobs);
    } catch (err) {
      if (jobs.length === 0) status = "error";
      error = String(err);
    } finally {
      if (isManual) isRefreshing = false;
    }
  }

  async function cancelJob(id: number) {
    try {
      await invoke("download_cancel", { id });
      await refresh();
    } catch (err) {
      error = String(err);
    }
  }

  async function deleteJob(id: number) {
    try {
      await invoke("download_delete", { id });
      await refresh();
    } catch (err) {
      error = String(err);
    }
  }

  async function clearFinished() {
    try {
      await invoke("downloads_clear");
      await refresh();
    } catch (err) {
      error = String(err);
    }
  }

  const hasFinishedJobs = $derived(
    jobs.some(
      (j) =>
        j.status === "dispatched" ||
        j.status === "completed" ||
        j.status === "failed"
    )
  );

  onMount(() => {
    refresh();
  });
  onDestroy(() => {
    if (timer) {
      clearInterval(timer);
      timer = undefined;
    }
  });

  function fmtTime(secs: number): string {
    return new Date(secs * 1000).toLocaleTimeString();
  }

  function platformLabel(p: string): string {
    const map: Record<string, string> = {
      pc: "PC",
      android: "Android",
      mac: "Mac",
      linux: "Linux",
    };
    return map[p.toLowerCase()] ?? p;
  }

  function fmtBytes(n: number): string {
    if (n < 1024) return `${n} B`;
    const units = ["KB", "MB", "GB", "TB"];
    let value = n / 1024;
    let i = 0;
    while (value >= 1024 && i < units.length - 1) {
      value /= 1024;
      i++;
    }
    return `${value.toFixed(value >= 10 ? 0 : 1)} ${units[i]}`;
  }

  function progressPct(job: QueueJob): number {
    if (job.bytes_total <= 0) return 0;
    return Math.min(100, Math.round((job.bytes_done * 100) / job.bytes_total));
  }
</script>

<section class="downloads">
  <div class="head">
    <h1>
      <span class="head-icon" aria-hidden="true"
        ><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3v12"/><path d="m6 11 6 6 6-6"/><path d="M4 21h16"/></svg></span
      >Downloads
    </h1>
    <div class="head-actions">
      <button
        type="button"
        class="refresh-btn"
        class:spinning={isRefreshing}
        disabled={isRefreshing}
        onclick={() => refresh(true)}
        title="Refresh downloads queue"
        aria-label="Refresh downloads queue"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67"/>
        </svg>
        <span>{isRefreshing ? "Refreshing…" : "Refresh"}</span>
      </button>
      {#if hasFinishedJobs}
        <button class="clear-btn" onclick={clearFinished}>
          Clear Finished
        </button>
      {/if}
    </div>
  </div>

  {#if status === "loading"}
    <p class="note">Loading queue…</p>
  {:else if status === "error"}
    <p class="note error">{error}</p>
  {:else if jobs.length === 0}
    <p class="note">
      No active downloads. Start one from the Store and it will appear here.
    </p>
  {:else}
    <div class="list" role="list">
      {#each jobs as job (job.id)}
        <div class="item" role="listitem">
          <div class="item-top">
            <span class="item-name">{cleanDisplayTitle(job.title || job.slug)}</span>
            <div class="item-actions">
              <span class="badge {job.status}">{statusLabel[job.status]}</span>
              {#if job.status === "queued" || job.status === "resolving" || job.status === "downloading" || job.status === "dispatching" || job.status === "extracting"}
                <button
                  class="action-btn cancel"
                  onclick={() => cancelJob(job.id)}
                  title="Cancel download"
                  aria-label={`Cancel download for ${cleanDisplayTitle(job.title || job.slug)}`}
                >
                  Cancel
                </button>
                <button
                  class="action-btn delete"
                  onclick={() => deleteJob(job.id)}
                  title="Remove from queue"
                  aria-label={`Remove ${cleanDisplayTitle(job.title || job.slug)} from downloads`}
                >
                  ✕
                </button>
              {:else if job.status === "dispatched" || job.status === "completed" || job.status === "failed"}
                <button
                  class="action-btn delete"
                  onclick={() => deleteJob(job.id)}
                  title="Remove from queue"
                  aria-label={`Remove ${cleanDisplayTitle(job.title || job.slug)} from downloads`}
                >
                  ✕
                </button>
              {/if}
            </div>
          </div>
          <div class="item-meta">
            <span>#{job.id}</span>
            <span>·</span>
            <span>{job.version}</span>
            <span>·</span>
            <span>{platformLabel(job.platform)}</span>
            {#if job.tab}
              <span>·</span>
              <span>{job.tab}</span>
            {/if}
            {#if job.source}
              <span>·</span>
              <span>{job.source}</span>
            {/if}
          </div>
          {#if job.title && formatCanonicalTitle(job.title) !== cleanDisplayTitle(job.title)}
            <p class="item-canonical">{formatCanonicalTitle(job.title)}</p>
          {/if}
          {#if job.status === "downloading" || job.status === "extracting"}
            <div
              class="bar"
              role="progressbar"
              aria-valuemin="0"
              aria-valuemax="100"
              aria-valuenow={progressPct(job)}
              aria-label={`${job.status === "extracting" ? "Extraction" : "Download"} progress for ${cleanDisplayTitle(job.title || job.slug)}`}
            >
              <div
                class="bar-fill"
                class:extracting={job.status === "extracting"}
                style={`width: ${progressPct(job)}%`}
              ></div>
            </div>
            <p class="item-bytes">
              {fmtBytes(job.bytes_done)}
              {#if job.bytes_total > 0}
                / {fmtBytes(job.bytes_total)} · {progressPct(job)}% {job.status === "extracting" ? "extracted" : "downloaded"}
              {:else}
                {job.status === "extracting" ? "extracted" : "downloaded"}
              {/if}
            </p>
          {/if}
          {#if job.message && job.message !== job.title}
            <p class="item-msg" class:error={job.status === "failed"}>{formatDisplayMessage(job.message)}</p>
          {/if}
          <p class="item-time">
            queued {fmtTime(job.created_at)}
            {#if job.updated_at !== job.created_at}
              · updated {fmtTime(job.updated_at)}
            {/if}
          </p>
        </div>
      {/each}
    </div>
  {/if}
</section>



<style>
  .downloads {
    padding: var(--lz-gap);
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
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

  .head-actions {
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
    font-size: 12px;
    font-weight: 500;
    padding: 5px 12px;
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

  .clear-btn {
    appearance: none;
    border: 1px solid var(--lz-surface-2);
    background: var(--lz-surface-2);
    color: var(--lz-text-dim);
    font: inherit;
    font-size: 12px;
    font-weight: 600;
    padding: 5px 12px;
    border-radius: var(--lz-radius);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .clear-btn:hover {
    color: #fff;
    background: rgba(255, 75, 75, 0.2);
    border-color: rgba(255, 75, 75, 0.4);
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

  .list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .item {
    background: var(--lz-surface);
    border: 1px solid var(--lz-surface-2);
    border-radius: var(--lz-radius);
    padding: 10px 12px;
  }

  .item-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  .item-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .action-btn {
    appearance: none;
    border: none;
    font: inherit;
    font-size: 11px;
    font-weight: 600;
    padding: 3px 8px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .action-btn.cancel {
    background: rgba(255, 120, 0, 0.15);
    color: #ffaa44;
    border: 1px solid rgba(255, 120, 0, 0.3);
  }

  .action-btn.cancel:hover {
    background: #ff8800;
    color: #0f1117;
  }

  .action-btn.delete {
    background: rgba(255, 75, 75, 0.15);
    color: #ff6b6b;
    border: 1px solid rgba(255, 75, 75, 0.3);
    padding: 2px 6px;
  }

  .action-btn.delete:hover {
    background: #ff4b4b;
    color: #fff;
  }

  .item-name {
    font-weight: 600;
    font-size: 14px;
  }

  .item-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 6px;
    color: var(--lz-text-dim);
    font-size: 12px;
  }

  .item-canonical,
  .item-msg {
    margin: 8px 0 0;
    font-size: 13px;
    color: var(--lz-text-dim);
    line-height: 1.4;
  }

  .bar {
    margin-top: 8px;
    height: 6px;
    border-radius: 4px;
    background: var(--lz-surface-2);
    overflow: hidden;
  }

  .bar-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--lz-primary), var(--lz-accent));
    transition: width 0.3s ease;
  }

  .bar-fill.extracting {
    background: linear-gradient(90deg, #ff9e00, var(--lz-accent));
  }

  .item-bytes {
    margin: 4px 0 0;
    font-size: 11px;
    color: var(--lz-text-dim);
  }

  .item-msg.error {
    color: var(--lz-danger);
  }

  .item-time {
    margin: 6px 0 0;
    font-size: 11px;
    color: var(--lz-text-dim);
  }

  .badge {
    font-size: 11px;
    padding: 2px 10px;
    border-radius: 10px;
    background: var(--lz-surface-2);
    color: var(--lz-text-dim);
    white-space: nowrap;
  }

  .badge.resolving,
  .badge.dispatching,
  .badge.downloading,
  .badge.extracting {
    color: var(--lz-accent);
  }

  .badge.dispatched,
  .badge.completed {
    color: var(--lz-cyan);
  }

  .badge.failed {
    color: var(--lz-danger);
  }
</style>