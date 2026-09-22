<svelte:options runes={true} />

<script lang="ts">
  import { onMount } from "svelte";

  type LoadState = "loading" | "ready" | "error";

  let status: LoadState = $state("loading");
  let error = $state("");

  onMount(async () => {
    try {
      // queue state arrives via the core download/list pipeline (Phase 2)
      status = "ready";
    } catch (err) {
      status = "error";
      error = String(err);
    }
  });
</script>

{#if status === "loading"}
  <p class="note">Loading queue…</p>
{:else if status === "error"}
  <p class="note error">{error}</p>
{:else}
  <p class="note">No active downloads. Start one from the Store.</p>
{/if}

<style>
  .note {
    padding: var(--lz-gap);
    color: var(--lz-text-dim);
  }

  .note.error {
    color: var(--lz-danger);
  }
</style>