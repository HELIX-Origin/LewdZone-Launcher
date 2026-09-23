<svelte:options runes={true} />

<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { loadAndApplyTheme } from "$lib/theme/apply";
  import "../lib/theme/default.css";

  let { children } = $props();

  const compact = () => getCurrentWindow().minimize();
  const zoom = () => getCurrentWindow().toggleMaximize();
  const quit = () => getCurrentWindow().close();

  // macOS convention puts the traffic lights on the left; Windows/Linux put
  // them on the right.
  const isMac = $derived(
    typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.userAgent),
  );

  // Drag the frameless window from anywhere in the bar except interactive
  // controls. `data-tauri-drag-region` on the whole bar would swallow every
  // child click, so the drag is started manually on background pointerdown.
  function onBarPointerDown(e: PointerEvent) {
    if (e.button !== 0) return;
    const t = e.target as HTMLElement | null;
    if (t?.closest("button, select, input, a, .menu-pop, [role='menuitem']")) return;
    e.preventDefault();
    getCurrentWindow().startDragging();
  }

  function onBarDoubleClick(e: MouseEvent) {
    const t = e.target as HTMLElement | null;
    if (t?.closest("button, select, input, a, .menu-pop, [role='menuitem']")) return;
    getCurrentWindow().toggleMaximize();
  }

  let openMenu = $state<string | null>(null);
  let aboutOpen = $state(false);

  const menus = [
    {
      id: "file",
      label: "File",
      items: [
        { label: "Store", action: () => goto("/store") },
        { label: "Favorites", action: () => goto("/favorites") },
        { label: "Library", action: () => goto("/library") },
        { label: "Settings", action: () => goto("/settings") },
        { label: "Quit", action: quit },
      ],
    },
    {
      id: "view",
      label: "View",
      items: [
        { label: "Store", action: () => goto("/store") },
        { label: "Favorites", action: () => goto("/favorites") },
        { label: "Library", action: () => goto("/library") },
        { label: "Downloads", action: () => goto("/downloads") },
      ],
    },
    {
      id: "help",
      label: "Help",
      items: [{ label: "About LewdZone Launcher", action: () => (aboutOpen = true) }],
    },
  ] as const;

  function toggleMenu(id: string) {
    openMenu = openMenu === id ? null : id;
  }

  function runItem(fn: () => void) {
    openMenu = null;
    fn();
  }

  const nav = [
    { id: "store", label: "Store", icon: "store", path: "/store" },
    { id: "favorites", label: "Favorites", icon: "favorites", path: "/favorites" },
    { id: "library", label: "Library", icon: "library", path: "/library" },
    { id: "settings", label: "Settings", icon: "settings", path: "/settings" },
  ] as const;

  const icons: Record<string, string> = {
    store:
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M4 4h16l2 5H2l2-5Z"/><path d="M4 9v10h16V9"/><path d="M9 19v-5h6v5"/></svg>',
    favorites:
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 20.5 4.6 13a4.8 4.8 0 0 1 0-6.9 5.1 5.1 0 0 1 7.4 0l.6.6.6-.6a5.1 5.1 0 0 1 7.4 0 4.8 4.8 0 0 1 0 6.9L12 20.5Z"/></svg>',
    library:
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M5 19V5h4v14H5Z"/><path d="M10 19V5h4v14h-4Z"/><path d="M15 19V5h4v14h-4Z"/></svg>',
    settings:
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="12" r="3.2"/><path d="M19.2 12a7.3 7.3 0 0 0-.1-1.2l1.9-1.5-2-3.4-2.2.9a7.4 7.4 0 0 0-2-1.2L14.5 3h-3.9l-.3 2.6a7.4 7.4 0 0 0-2 1.2l-2.2-.9-2 3.4 1.9 1.5a7.3 7.3 0 0 0 0 2.4L4.1 14.7l2 3.4 2.2-.9a7.4 7.4 0 0 0 2 1.2l.3 2.6h3.9l.3-2.6a7.4 7.4 0 0 0 2-1.2l2.2.9 2-3.4-1.9-1.5c.1-.4.1-.8.1-1.2Z"/></svg>',
  };

  const current = $derived(page.url.pathname);

  function isActive(item: (typeof nav)[number]): boolean {
    return current === item.path;
  }

  function go(item: (typeof nav)[number]) {
    goto(item.path);
  }

  onMount(() => {
    loadAndApplyTheme();
  });
</script>

<div class="app">
  <header
    class:mac={isMac}
    class="titlebar"
    role="presentation"
    onpointerdown={onBarPointerDown}
    ondblclick={onBarDoubleClick}
  >
    <div class="traffic" aria-label="Window controls">
      <button class="dot close" aria-label="Close window" onclick={quit}></button>
      <button class="dot min" aria-label="Minimize window" onclick={compact}></button>
      <button class="dot max" aria-label="Maximize window" onclick={zoom}></button>
    </div>
    <nav class="menubar" aria-label="Application menu">
      {#each menus as menu (menu.id)}
        <div class="menu">
          <button
            class="menu-trigger"
            aria-haspopup="menu"
            aria-expanded={openMenu === menu.id}
            onclick={() => toggleMenu(menu.id)}
          >
            {menu.label}
          </button>
          {#if openMenu === menu.id}
            <div class="menu-pop" role="menu">
              {#each menu.items as item (item.label)}
                <button
                  class="menu-item"
                  role="menuitem"
                  onclick={() => runItem(item.action)}
                >
                  {item.label}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </nav>
    <span class="tb-title">LewdZone Launcher</span>
    <div class="spacer"></div>
    <button class="avatar" aria-label="Profile">
      <span aria-hidden="true" class="avatar-icon"
        ><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="8" r="3.8"/><path d="M4.5 20a7.5 7.5 0 0 1 15 0"/></svg></span
      >
    </button>
  </header>

  {#if openMenu}
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="menu-backdrop" role="none" onclick={() => (openMenu = null)}></div>
  {/if}
  {#if aboutOpen}
    <div class="about-backdrop">
      <div class="about-card" role="dialog" aria-label="About">
        <h2>LewdZone Launcher</h2>
        <p>A cross-platform desktop launcher + CLI engine for LewdZone.</p>
        <button class="about-close" onclick={() => (aboutOpen = false)}>Close</button>
      </div>
    </div>
  {/if}

  <div class="shell">
    <aside class="sidebar" aria-label="Primary">
    <img src="/favicon.png" alt="" class="logo" />
    {#each nav as item (item.id)}
      <button
        class="nav-btn"
        class:active={isActive(item)}
        onclick={() => go(item)}
        aria-label={item.label}
        aria-current={isActive(item) ? "page" : undefined}
        title={item.label}
      >
        <span class="nav-icon" role="presentation">{@html icons[item.icon]}</span>
      </button>
    {/each}
  </aside>

  <div class="main">
    <main class="content">
      {@render children?.()}
    </main>
  </div>
  </div>
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--lz-gradient);
    color: var(--lz-text);
  }

  .titlebar {
    position: relative;
    z-index: 10;
    flex: 0 0 auto;
    height: 36px;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 12px;
    background: var(--lz-glass);
    backdrop-filter: blur(10px);
    border-bottom: 1px solid var(--lz-surface-2);
    user-select: none;
    cursor: grab;
  }

  .menubar {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .traffic {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  /* Default (Windows/Linux): traffic lights on the right, after the avatar. */
  .titlebar .menubar {
    order: 1;
  }

  .titlebar .tb-title {
    order: 2;
  }

  .titlebar .spacer {
    order: 3;
  }

  .titlebar .avatar {
    order: 4;
  }

  .titlebar .traffic {
    order: 5;
  }

  /* macOS: traffic lights on the left, before the menu bar. */
  .titlebar.mac .traffic {
    order: 0;
    margin-right: 2px;
  }

  .menu {
    position: relative;
  }

  .menu-trigger {
    appearance: none;
    border: none;
    background: transparent;
    color: var(--lz-text-dim);
    font: inherit;
    font-size: 12px;
    padding: 4px 10px;
    border-radius: var(--lz-radius);
    cursor: pointer;
  }

  .menu-trigger:hover,
  .menu-trigger[aria-expanded="true"] {
    color: var(--lz-text);
    background: var(--lz-surface-2);
  }

  .menu-pop {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    min-width: 200px;
    background: var(--lz-surface);
    border: 1px solid var(--lz-surface-2);
    border-radius: var(--lz-radius);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
    padding: 4px;
    display: flex;
    flex-direction: column;
    z-index: 30;
  }

  .menu-item {
    appearance: none;
    border: none;
    background: transparent;
    color: var(--lz-text);
    font: inherit;
    font-size: 12px;
    text-align: left;
    padding: 6px 10px;
    border-radius: calc(var(--lz-radius) - 1px);
    cursor: pointer;
  }

  .menu-item:hover {
    background: var(--lz-surface-2);
  }

  .menu-backdrop {
    position: fixed;
    inset: 0;
    z-index: 5;
  }

  .spacer {
    flex: 1 1 auto;
  }

.avatar {
    appearance: none;
    border: 1px solid var(--lz-surface-2);
    background: var(--lz-surface-2);
    color: var(--lz-cyan);
    font: inherit;
    font-size: 13px;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .avatar-icon :global(svg) {
    width: 16px;
    height: 16px;
    display: block;
  }

  .avatar:hover {
    box-shadow: var(--lz-glow);
  }

  .about-backdrop {
    position: fixed;
    inset: 0;
    z-index: 40;
    background: rgba(5, 10, 14, 0.55);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .about-card {
    background: var(--lz-surface);
    border: 1px solid var(--lz-surface-2);
    border-radius: var(--lz-radius);
    box-shadow: 0 12px 36px rgba(0, 0, 0, 0.5);
    padding: 20px;
    max-width: 320px;
    text-align: center;
  }

  .about-card h2 {
    margin: 0 0 8px;
    font-size: 16px;
    color: var(--lz-cyan);
  }

  .about-card p {
    color: var(--lz-text-dim);
    font-size: 13px;
    line-height: 1.5;
    margin: 0 0 14px;
  }

  .about-close {
    appearance: none;
    border: none;
    background: linear-gradient(135deg, var(--lz-primary), var(--lz-accent));
    color: #fff;
    font: inherit;
    font-size: 12px;
    font-weight: 700;
    padding: 6px 16px;
    border-radius: var(--lz-radius);
    cursor: pointer;
  }

  .traffic {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .dot {
    width: 13px;
    height: 13px;
    border-radius: 50%;
    border: none;
    padding: 0;
    cursor: pointer;
    position: relative;
  }

  /* Windows/Linux order: minimize, maximize, close (left → right). */
  .traffic .dot.min {
    order: 1;
  }

  .traffic .dot.max {
    order: 2;
  }

  .traffic .dot.close {
    order: 3;
  }

  /* macOS order: close, minimize, maximize (left → right). */
  .titlebar.mac .traffic .dot.close {
    order: 1;
  }

  .titlebar.mac .traffic .dot.min {
    order: 2;
  }

  .titlebar.mac .traffic .dot.max {
    order: 3;
  }

  .dot.close {
    background: #ff5f57;
  }

  .dot.min {
    background: #febc2e;
  }

  .dot.max {
    background: #28c840;
  }

  .dot:hover::after {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.25);
  }

  .tb-title {
    font-size: 12px;
    color: var(--lz-text-dim);
  }

  .shell {
    display: flex;
    flex: 1 1 auto;
    min-height: 0;
    background: transparent;
    color: var(--lz-text);
  }

  .sidebar {
    width: 72px;
    flex: 0 0 auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 12px 6px;
    background: var(--lz-glass);
    backdrop-filter: blur(10px);
    border-right: 1px solid var(--lz-surface-2);
    z-index: 2;
  }

  .logo {
    width: 36px;
    height: 36px;
    border-radius: 8px;
    margin-bottom: 10px;
    box-shadow: var(--lz-glow);
  }

  .nav-btn {
    appearance: none;
    border: none;
    background: transparent;
    color: var(--lz-text-dim);
    font: inherit;
    width: 60px;
    padding: 6px 2px;
    border-radius: var(--lz-radius);
    cursor: pointer;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    transition:
      color 0.15s ease,
      background 0.15s ease,
      box-shadow 0.15s ease;
  }

  .nav-btn:hover {
    color: var(--lz-text);
    background: var(--lz-surface-2);
  }

  .nav-btn.active {
    color: var(--lz-cyan);
    background: var(--lz-surface-2);
    box-shadow: var(--lz-glow);
  }

  .nav-icon {
    width: 20px;
    height: 20px;
    line-height: 1;
  }

  .nav-icon :global(svg) {
    width: 20px;
    height: 20px;
    display: block;
  }

  .main {
    flex: 1 1 auto;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .content {
    flex: 1 1 auto;
    overflow: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--lz-surface-2) transparent;
  }
</style>