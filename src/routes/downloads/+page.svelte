<svelte:options runes={true} />

<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  type LoadState = "loading" | "ready" | "error";

  type Status = "queued" | "resolving" | "dispatching" | "dispatched" | "failed";

  interface QueueJob {
    id: number;
    slug: string;
    version: string;
    platform: string;
    tab: string;
    source: string | null;
    status: Status;
    message: string | null;
    manager: string | null;
    created_at: number;
    updated_at: number;
  }

  const statusLabel: Record<Status, string> = {
    queued: "Queued",
    resolving: "Resolving",
    dispatching: "Dispatching",
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
</script>

<section class="downloads">
  <div class="head">
    <h1>
      <span class="head-icon" aria-hidden="true"
        ><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3v12"/><path d="m6 11 6 6 6-6"/><path d="M4 21h16"/></svg></span
      >Downloads
    </h1>
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
            <span class="badge {job.status}">{statusLabel[job.status]}</span>
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
            {#if job.manager}
              <span>·</span>
              <span>{job.manager}</span>
            {/if}
          </div>
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
  .badge.dispatching {
    color: var(--lz-accent);
  }

  .badge.dispatched {
    color: var(--lz-cyan);
  }

  .badge.failed {
    color: var(--lz-danger);
  }
</style>