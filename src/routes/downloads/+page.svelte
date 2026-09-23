<svelte:options runes={true} />

<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  type LoadState = "loading" | "ready" | "error";

  type Status =
    | "queued"
    | "resolving"
    | "dispatching"
    | "downloading"
    | "dispatched"
    | "failed";

  interface QueueJob {
    id: number;
    slug: string;
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
    dispatched: "Dispatched",
    failed: "Failed",
  };

  let status: LoadState = $state("loading");
  let error = $state("");
  let jobs: QueueJob[] = $state([]);

  async function refresh() {
    try {
      jobs = await invoke<QueueJob[]>("downloads_list");
      status = "ready";
    } catch (err) {
      status = "error";
      error = String(err);
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
    jobs.some((j) => j.status === "dispatched" || j.status === "failed")
  );

  let timer: ReturnType<typeof setInterval> | undefined;
  onMount(() => {
    refresh();
    timer = setInterval(refresh, 1000);
  });
  onDestroy(() => {
    if (timer) clearInterval(timer);
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
    {#if hasFinishedJobs}
      <button class="clear-btn" onclick={clearFinished}>
        Clear Finished
      </button>
    {/if}
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
            <span class="item-name">{job.slug}</span>
            <div class="item-actions">
              <span class="badge {job.status}">{statusLabel[job.status]}</span>
              {#if job.status === "queued" || job.status === "resolving" || job.status === "downloading" || job.status === "dispatching"}
                <button
                  class="action-btn cancel"
                  onclick={() => cancelJob(job.id)}
                  title="Cancel download"
                  aria-label={`Cancel download for ${job.slug}`}
                >
                  Cancel
                </button>
              {:else if job.status === "dispatched" || job.status === "failed"}
                <button
                  class="action-btn delete"
                  onclick={() => deleteJob(job.id)}
                  title="Remove from queue"
                  aria-label={`Remove ${job.slug} from downloads`}
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
          {#if job.status === "downloading"}
            <div
              class="bar"
              role="progressbar"
              aria-valuemin="0"
              aria-valuemax="100"
              aria-valuenow={progressPct(job)}
              aria-label={`Download progress for ${job.slug}`}
            >
              <div class="bar-fill" style={`width: ${progressPct(job)}%`}></div>
            </div>
            <p class="item-bytes">
              {fmtBytes(job.bytes_done)}
              {#if job.bytes_total > 0}
                / {fmtBytes(job.bytes_total)} · {progressPct(job)}%
              {:else}
                downloaded
              {/if}
            </p>
          {/if}
          {#if job.message}
            <p class="item-msg" class:error={job.status === "failed"}>{job.message}</p>
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
  .badge.downloading {
    color: var(--lz-accent);
  }

  .badge.dispatched {
    color: var(--lz-cyan);
  }

  .badge.failed {
    color: var(--lz-danger);
  }
</style>