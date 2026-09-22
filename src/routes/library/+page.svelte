<svelte:options runes={true} />

<script lang="ts">
  import { onMount } from "svelte";

  type LoadState = "loading" | "ready" | "error";

  let status: LoadState = $state("loading");
  let error = $state("");

  onMount(async () => {
    try {
      // installed games derive from appmanifests (core::library, Phase 2)
      status = "ready";
    } catch (err) {
      status = "error";
      error = String(err);
    }
  });
</script>

{#if status === "loading"}
  <p class="note">Reading your library…</p>
{:else if status === "error"}
  <p class="note error">{error}</p>
{:else}
  <p class="note">Nothing installed yet. Games you download will appear here.</p>
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