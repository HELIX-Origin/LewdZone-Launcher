<svelte:options runes={true} />

<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { loadAndApplyTheme } from "$lib/theme/apply";
  import "../lib/theme/default.css";

  let { children } = $props();

  const tabs = [
    { id: "store", label: "Store", path: "/store" },
    { id: "library", label: "Library", path: "/library" },
    { id: "downloads", label: "Downloads", path: "/downloads" },
    { id: "settings", label: "Settings", path: "/settings" },
  ] as const;

  const current = $derived(page.url.pathname);

  function isActive(tab: (typeof tabs)[number]): boolean {
    if (tab.id === "store") return current === "/" || current === "/store";
    return current === tab.path;
  }

  onMount(() => {
    loadAndApplyTheme();
  });
</script>

<div class="shell">
  <header class="topbar">
    <div class="brand">
      <img src="/favicon.png" alt="" class="brand-icon" />
      <span class="brand-name">LewdZone</span>
    </div>
    <nav class="tabs" aria-label="Primary">
      {#each tabs as tab (tab.id)}
        <button
          class="tab"
          class:active={isActive(tab)}
          onclick={() => goto(tab.path)}
          aria-current={isActive(tab) ? "page" : undefined}
        >
          {tab.label}
        </button>
      {/each}
    </nav>
  </header>

  <main class="content">
    {@render children?.()}
  </main>
</div>

<style>
  .shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--lz-bg);
    color: var(--lz-text);
  }

  .topbar {
    display: flex;
    align-items: center;
    gap: var(--lz-gap);
    padding: 10px var(--lz-gap);
    background: var(--lz-surface);
    border-bottom: 1px solid var(--lz-surface-2);
    flex: 0 0 auto;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-right: 16px;
  }

  .brand-icon {
    width: 20px;
    height: 20px;
    border-radius: 4px;
  }

  .brand-name {
    font-weight: 700;
    letter-spacing: 0.02em;
  }

  .tabs {
    display: flex;
    gap: 4px;
  }

  .tab {
    appearance: none;
    border: none;
    background: transparent;
    color: var(--lz-text-dim);
    font: inherit;
    padding: 6px 14px;
    border-radius: var(--lz-radius);
    cursor: pointer;
    transition:
      color 0.15s ease,
      background 0.15s ease;
  }

  .tab:hover {
    color: var(--lz-text);
    background: var(--lz-surface-2);
  }

  .tab.active {
    color: var(--lz-accent);
    background: var(--lz-surface-2);
    font-weight: 600;
  }

  .content {
    flex: 1 1 auto;
    overflow: auto;
  }
</style>