<svelte:options runes={true} />

<script lang="ts">
  import { onMount } from "svelte";

  type LoadState = "loading" | "ready" | "error";

  let status: LoadState = $state("loading");
  let error = $state("");

  onMount(async () => {
    try {
      // catalog arrives via the core `sync`/`search` pipeline (Phase 2)
      status = "ready";
    } catch (err) {
      status = "error";
      error = String(err);
    }
  });

  const tiles = Array.from({ length: 8 });
</script>

{#if status === "loading"}
  <p class="note">Loading catalog…</p>
{:else if status === "error"}
  <p class="note error">{error}</p>
{:else}
  <div class="grid" role="list" aria-label="Game catalog">
    {#each tiles as _, i (i)}
      <div class="tile" role="listitem">
        <div class="cover" aria-hidden="true"></div>
        <div class="tile-title">Game title {i + 1}</div>
        <div class="tile-meta">PC · Ongoing</div>
      </div>
    {/each}
  </div>
{/if}

<style>
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
    opacity: 0.55;
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