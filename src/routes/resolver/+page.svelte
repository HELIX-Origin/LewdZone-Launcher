<svelte:options runes={true} />

<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { emit } from "@tauri-apps/api/event";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { page } from "$app/state";
  import { formatCanonicalTitle } from "$lib/format";

  interface ResolvedUrl {
    url: string;
    host: string;
    platform: string | null;
    version: string | null;
    game_id: number | null;
  }

  type Status = "connecting" | "countdown" | "revealing" | "resolved" | "error";

  let status: Status = $state("connecting");
  let errorMessage = $state("");
  let resolved: ResolvedUrl | null = $state(null);
  let countdownSecs = $state(5);
  let copyFeedback = $state(false);

  const slug = $derived(page.url.searchParams.get("slug") ?? "game");
  const goLink = $derived(page.url.searchParams.get("url") ?? "");
  const host = $derived(page.url.searchParams.get("host") ?? "");
  const rawTitle = $derived(page.url.searchParams.get("title") ?? slug);
  const title = $derived(formatCanonicalTitle(rawTitle));

  const HOST_ICONS: Record<string, string> = {
    mega: `<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="11" fill="#D9272E"/><path d="M7 16V8l5 4.5L17 8v8h-2v-5l-3 2.7-3-2.7V16H7z" fill="#FFF"/></svg>`,
    google: `<svg viewBox="0 0 24 24"><path d="M8.5 3.5l-6 10.5 3.5 6 6-10.5z" fill="#0066DA"/><path d="M15.5 3.5H8.5l6 10.5h7z" fill="#00AC47"/><path d="M21.5 14H9.5l-3.5 6h12z" fill="#EA4335"/><path d="M6 14l3.5-6-3.5-6L0 12z" fill="#FFBA00"/></svg>`,
    dropbox: `<svg viewBox="0 0 24 24"><path d="M6 3.5L0 7.5l6 4 6-4-6-4zm12 0l-6 4 6 4 6-4-6-4zM0 15.5l6 4 6-4-6-4-6 4zm24 0l-6-4-6 4 6 4 6-4zM6 21l6-4 6 4-6 4-6-4z" fill="#0061FF"/></svg>`,
    mediafire: `<svg viewBox="0 0 24 24"><path d="M17.5 10.5c-.5-3.5-3.5-6-7-6-2.5 0-4.8 1.4-5.9 3.5C2 8.5 0 11 0 14c0 3.3 2.7 6 6 6h11.5c3.6 0 6.5-2.9 6.5-6.5 0-3.3-2.5-6-5.8-6.5z" fill="#1299F3"/><path d="M12 7c-1.5 2-2.5 4-2.5 6 0 2 1.5 3.5 3.5 3.5s3.5-1.5 3.5-3.5c0-2-1.5-4-3-6-.3-.4-.8-.4-1.5 0z" fill="#FFF"/></svg>`,
    pixeldrain: `<svg viewBox="0 0 24 24"><path d="M12 2C8 7 5 11 5 15a7 7 0 0 0 14 0c0-4-3-8-7-13z" fill="#FF6B4A"/><circle cx="10" cy="14" r="2.5" fill="#FFF"/></svg>`,
    fileknot: `<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="#7928CA"/><path d="M8 8a4 4 0 0 1 8 0v8a4 4 0 0 1-8 0V8z" fill="none" stroke="#FFF" stroke-width="2.2"/><circle cx="12" cy="12" r="2.2" fill="#00DFD8"/></svg>`,
    transfaze: `<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="#00C08B"/><path d="M7 10l5-4 5 4M17 14l-5 4-5-4" stroke="#FFF" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" fill="none"/></svg>`,
    workupload: `<svg viewBox="0 0 24 24"><rect x="3" y="4" width="18" height="16" rx="4" fill="#2ECC71"/><path d="M12 8v8M8 12l4-4 4 4" stroke="#FFF" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
    uploadhaven: `<svg viewBox="0 0 24 24"><path d="M12 2L3 6v6c0 5.5 3.8 10.7 9 12 5.2-1.3 9-6.5 9-12V6l-9-4z" fill="#0E76A8"/><path d="M12 7l4 4h-3v5h-2v-5H8l4-4z" fill="#FFF"/></svg>`,
    "1fichier": `<svg viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="4" fill="#FF6600"/><path d="M10 8l3-2v12h-3" fill="none" stroke="#FFF" stroke-width="2.8" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
    rapidgator: `<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="#E65100"/><path d="M9 7h4a3 3 0 0 1 0 6H9zm0 6l5 5" stroke="#FFF" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" fill="none"/></svg>`,
    qiwi: `<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="#FF8C00"/><circle cx="12" cy="12" r="5.5" fill="#FFF"/><circle cx="13.5" cy="10.5" r="1.8" fill="#FF8C00"/></svg>`,
    bowfile: `<svg viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="4" fill="#3B82F6"/><path d="M7 17L17 7M7 7h6M17 17V11" stroke="#FFF" stroke-width="2" stroke-linecap="round"/></svg>`,
    hexload: `<svg viewBox="0 0 24 24"><polygon points="12 2 21 7 21 17 12 22 3 17 3 7" fill="#8B5CF6"/><polyline points="8 12 12 16 16 8" stroke="#FFF" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none"/></svg>`,
    mixdrop: `<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="#EC4899"/><path d="M12 6v8M9 11l3 3 3-3" stroke="#FFF" stroke-width="2" stroke-linecap="round" fill="none"/></svg>`,
    racaty: `<svg viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="4" fill="#10B981"/><path d="M7 12h10M12 7v10" stroke="#FFF" stroke-width="2" stroke-linecap="round"/></svg>`,
    terminal: `<svg viewBox="0 0 24 24"><rect x="2" y="4" width="20" height="16" rx="3" fill="#1E293B"/><path d="M6 9l3 3-3 3M11 15h6" stroke="#10B981" stroke-width="2" stroke-linecap="round"/></svg>`,
  };

  function getHostIcon(h: string): string {
    const key = h.toLowerCase();
    return (
      HOST_ICONS[key] ??
      `<svg viewBox="0 0 24 24"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none"/></svg>`
    );
  }

  onMount(() => {
    if (!goLink) {
      status = "error";
      errorMessage = "No go-link token provided to resolver.";
      return;
    }
    startResolution();
  });

  async function startResolution() {
    status = "countdown";
    countdownSecs = 5;

    const timer = setInterval(() => {
      if (countdownSecs > 1) {
        countdownSecs -= 1;
      } else {
        clearInterval(timer);
        status = "revealing";
      }
    }, 1000);

    try {
      const result = await invoke<ResolvedUrl>("resolve_go_link", {
        goLink,
      });
      clearInterval(timer);
      resolved = result;
      status = "resolved";

      // Auto-notify launcher of resolved URL
      try {
        await emit("download-url-resolved", {
          slug,
          url: result.url,
        });
      } catch (e) {
        console.warn("Could not emit download-url-resolved:", e);
      }
    } catch (err) {
      clearInterval(timer);
      status = "error";
      errorMessage = String(err);
    }
  }

  async function copyUrl() {
    if (!resolved?.url) return;
    try {
      await navigator.clipboard.writeText(resolved.url);
      copyFeedback = true;
      setTimeout(() => {
        copyFeedback = false;
      }, 2000);
    } catch (err) {
      console.error("Clipboard copy failed:", err);
    }
  }

  async function openInApp() {
    if (!resolved?.url) return;
    const url = resolved.url;
    const lower = url.toLowerCase();
    const isDirectArchive = [
      ".zip", ".7z", ".rar", ".exe", ".tar.gz", ".tar.bz2", ".tar.xz", ".tgz", ".iso", ".apk", ".001", ".part1.rar", ".dmg", ".pkg"
    ].some((ext) => lower.includes(ext));

    if (isDirectArchive) {
      try {
        await invoke("queue_intercepted_download", {
          slug,
          url,
          title,
          version: resolved.version ?? "latest",
          platform: resolved.platform ?? "pc",
        });
        await emit("archive-intercepted", slug);
        await closeWindow();
        return;
      } catch (err) {
        console.warn("Direct enqueue failed, navigating in window:", err);
      }
    }

    window.location.href = url;
  }

  async function openExternal() {
    if (!resolved?.url) return;
    try {
      await openUrl(resolved.url);
    } catch (err) {
      window.open(resolved.url, "_blank");
    }
  }

  async function closeWindow() {
    try {
      const { getCurrentWebviewWindow } = await import("@tauri-apps/api/webviewWindow");
      const current = getCurrentWebviewWindow();
      await current.destroy();
    } catch (err) {
      window.close();
    }
  }
</script>

<div class="resolver-container">
  <header class="header">
    <div class="shield-badge">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
        <path d="m9 12 2 2 4-4"/>
      </svg>
      <span>Secure Ad-Free Resolver</span>
    </div>
    <h1>{title}</h1>
    {#if host}
      <span class="host-pill">
        <span class="host-icon">{@html getHostIcon(host)}</span>
        <span>{host.toUpperCase()}</span>
      </span>
    {/if}
  </header>

  <main class="content">
    {#if status === "countdown"}
      <div class="card pulse">
        <div class="spinner-ring">
          <span class="count">{countdownSecs}</span>
        </div>
        <h2>Bypassing Redirect Challenge</h2>
        <p class="subtitle">Ad networks blocked. Waiting for link verification countdown…</p>
        <div class="progress-bar">
          <div class="progress-fill" style={`width: ${((5 - countdownSecs) / 5) * 100}%`}></div>
        </div>
      </div>
    {:else if status === "revealing" || status === "connecting"}
      <div class="card pulse">
        <div class="spinner"></div>
        <h2>Unlocking Direct Download</h2>
        <p class="subtitle">Negotiating download token with server API…</p>
      </div>
    {:else if status === "resolved" && resolved}
      <div class="card success">
        <div class="success-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="20 6 9 17 4 12"></polyline>
          </svg>
        </div>
        <h2>Download Host Link Ready</h2>
        <p class="subtitle">Ad challenge bypassed. Open in-app to trigger the download, and the background service will capture the archive automatically.</p>

        <div class="url-box">
          <code>{resolved.url}</code>
        </div>

        <div class="action-grid">
          <button class="btn primary" onclick={openInApp}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
              <polyline points="7 10 12 15 17 10"></polyline>
              <line x1="12" y1="15" x2="12" y2="3"></line>
            </svg>
            Open in App
          </button>

          <button class="btn secondary" onclick={openExternal}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
              <polyline points="15 3 21 3 21 9"></polyline>
              <line x1="10" y1="14" x2="21" y2="3"></line>
            </svg>
            Open in Browser
          </button>

          <button class="btn secondary" onclick={copyUrl}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
            </svg>
            {copyFeedback ? "Copied!" : "Copy Link"}
          </button>

          <button class="btn dismiss" onclick={closeWindow}>
            Done
          </button>
        </div>
      </div>
    {:else if status === "error"}
      <div class="card error">
        <div class="error-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="8" x2="12" y2="12"></line>
            <line x1="12" y1="16" x2="12.01" y2="16"></line>
          </svg>
        </div>
        <h2>Resolution Challenge Failed</h2>
        <p class="error-msg">{errorMessage}</p>
        <div class="action-grid">
          <button class="btn primary" onclick={startResolution}>
            Retry
          </button>
          <button class="btn dismiss" onclick={closeWindow}>
            Close
          </button>
        </div>
      </div>
    {/if}
  </main>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    background: #0f1117;
    color: #e4e7eb;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    user-select: none;
    overflow: hidden;
  }

  .resolver-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    padding: 24px;
    box-sizing: border-box;
    background: radial-gradient(circle at top right, rgba(0, 238, 255, 0.08), transparent 40%),
                radial-gradient(circle at bottom left, rgba(255, 0, 85, 0.05), transparent 40%),
                #0f1117;
  }

  .header {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    margin-bottom: 20px;
  }

  .shield-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: 20px;
    background: rgba(0, 238, 255, 0.12);
    border: 1px solid rgba(0, 238, 255, 0.3);
    color: #00eeff;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.5px;
    text-transform: uppercase;
    margin-bottom: 10px;
  }

  .shield-badge svg {
    width: 14px;
    height: 14px;
  }

  .header h1 {
    font-size: 18px;
    margin: 0 0 8px;
    color: #fff;
    max-width: 90%;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .host-pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px;
    border-radius: 4px;
    background: #1a1d26;
    border: 1px solid #2a2e3d;
    color: #a0aec0;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 1px;
  }

  .host-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
  }

  .host-icon :global(svg) {
    width: 14px;
    height: 14px;
    display: block;
  }

  .content {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .card {
    background: #151821;
    border: 1px solid #262b3a;
    border-radius: 12px;
    padding: 32px 24px;
    width: 100%;
    max-width: 520px;
    text-align: center;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .card h2 {
    font-size: 18px;
    margin: 16px 0 6px;
    color: #fff;
  }

  .subtitle {
    color: #8892b0;
    font-size: 13px;
    margin: 0 0 20px;
    max-width: 400px;
    line-height: 1.4;
  }

  .spinner-ring {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    border: 3px solid rgba(0, 238, 255, 0.2);
    border-top-color: #00eeff;
    display: flex;
    align-items: center;
    justify-content: center;
    animation: rotate 1.5s linear infinite;
  }

  .spinner-ring .count {
    font-size: 24px;
    font-weight: 800;
    color: #00eeff;
    animation: counter-rotate 1.5s linear infinite;
  }

  @keyframes rotate {
    to { transform: rotate(360deg); }
  }

  @keyframes counter-rotate {
    to { transform: rotate(-360deg); }
  }

  .spinner {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    border: 3px solid rgba(0, 238, 255, 0.2);
    border-top-color: #00eeff;
    animation: rotate 0.8s linear infinite;
  }

  .progress-bar {
    width: 100%;
    max-width: 320px;
    height: 6px;
    background: #1f2433;
    border-radius: 3px;
    overflow: hidden;
    margin-top: 10px;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #00eeff, #ff0055);
    transition: width 0.3s ease;
  }

  .success-icon {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    background: rgba(0, 255, 136, 0.15);
    color: #00ff88;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .success-icon svg {
    width: 28px;
    height: 28px;
  }

  .error-icon {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    background: rgba(255, 75, 75, 0.15);
    color: #ff4b4b;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .error-icon svg {
    width: 28px;
    height: 28px;
  }

  .error-msg {
    color: #ff7575;
    background: rgba(255, 75, 75, 0.08);
    border: 1px solid rgba(255, 75, 75, 0.2);
    padding: 10px 14px;
    border-radius: 6px;
    font-size: 13px;
    max-width: 440px;
    word-break: break-word;
    margin-bottom: 20px;
  }

  .url-box {
    width: 100%;
    box-sizing: border-box;
    background: #0d0f15;
    border: 1px solid #1e2230;
    padding: 10px 14px;
    border-radius: 6px;
    margin-bottom: 20px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .url-box code {
    font-family: "JetBrains Mono", Consolas, monospace;
    font-size: 12px;
    color: #00eeff;
  }

  .action-grid {
    display: flex;
    gap: 10px;
    width: 100%;
    justify-content: center;
  }

  .btn {
    appearance: none;
    border: none;
    border-radius: 8px;
    padding: 10px 16px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    transition: transform 0.1s ease, filter 0.15s ease;
  }

  .btn:hover {
    filter: brightness(1.15);
  }

  .btn:active {
    transform: scale(0.98);
  }

  .btn svg {
    width: 15px;
    height: 15px;
  }

  .btn.primary {
    background: linear-gradient(135deg, #00eeff, #0077ff);
    color: #0f1117;
    font-weight: 700;
  }

  .btn.secondary {
    background: #202534;
    color: #e4e7eb;
    border: 1px solid #2d3449;
  }

  .btn.dismiss {
    background: transparent;
    color: #8892b0;
    border: 1px solid #242938;
  }

  .btn.dismiss:hover {
    color: #fff;
    background: #1a1e2b;
  }
</style>
