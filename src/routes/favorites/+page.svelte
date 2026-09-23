<svelte:options runes={true} />

<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  type LoadState = "loading" | "ready" | "error";
  let status: LoadState = $state("loading");
  let error = $state("");

  async function load() {
    status = "loading";
    try {
      // Favorites persistence (SQLite `sync_state` key) lands with the
      // library/downloads milestone; surface the tab now with an empty list.
      const rows = await invoke<unknown[]>("favorites_list");
      void rows;
      status = "ready";
    } catch (err) {
      status = "error";
      error = String(err);
    }
  }

  onMount(() => load());
</script>

{#if status === "loading"}
  <p class="note">Loading favorites…</p>
{:else if status === "error"}
  <p class="note error">{error}</p>
{:else}
  <section class="favorites">
    <h1>
      <span class="head-icon" aria-hidden="true"
        ><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20.5 4.6 13a4.8 4.8 0 0 1 0-6.9 5.1 5.1 0 0 1 7.4 0l.6.6.6-.6a5.1 5.1 0 0 1 7.4 0 4.8 4.8 0 0 1 0 6.9L12 20.5Z"/></svg></span
      >Favorites
    </h1>
    <p class="note">
      Games you favorite in the Store will appear here. Use the heart on any
      game tile or detail page to save it for quick access.
    </p>
    <div class="empty" role="list">
      <div class="empty-card" role="listitem">No favorites yet.</div>
    </div>
  </section>
{/if}

<style>
  .favorites {
    padding: var(--lz-gap);
  }

  .favorites h1 {
    font-size: 18px;
    margin: 0 0 8px;
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
</style>