<svelte:options runes={true} />

<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { page } from "$app/state";
  import { loadAndApplyTheme } from "$lib/theme/apply";
  import "$lib/theme/default.css";

  interface InstallerStatus {
    is_installed: boolean;
    installed_version: string | null;
    current_version: string;
    default_install_dir: string;
    current_exe_path: string;
    os: string;
  }

  interface DiskSpaceInfo {
    available_bytes: number;
    required_bytes: number;
    has_sufficient_space: boolean;
  }

  interface OperationResult {
    success: boolean;
    message: string;
    details: string[];
  }

  type WizardMode = "install" | "maintenance";
  type InstallTab = "welcome" | "destination" | "options" | "installing" | "complete";
  type MaintenanceTab = "manage" | "uninstall_options" | "removing" | "finished";

  let statusLoading = $state(true);
  let info = $state<InstallerStatus | null>(null);
  let mode = $state<WizardMode>("install");

  // Tab navigation
  let installTab = $state<InstallTab>("welcome");
  let maintenanceTab = $state<MaintenanceTab>("manage");

  // Install options
  let targetDir = $state("");
  let createDesktop = $state(true);
  let createStartMenu = $state(true);
  let addToPath = $state(true);
  let launchAfter = $state(true);

  // Maintenance options
  let removeUserData = $state(false);

  // Progress state
  let inProgress = $state(false);
  let progressPct = $state(0);
  let progressStep = $state("Preparing files…");
  let result = $state<OperationResult | null>(null);
  let logDetails = $state<string[]>([]);
  let showLogs = $state(false);

  // Window drag & controls
  const closeWindow = async () => {
    try {
      await getCurrentWindow().destroy();
    } catch {
      getCurrentWindow().close();
    }
  };
  const minimizeWindow = () => getCurrentWindow().minimize();
  const maximizeWindow = () => getCurrentWindow().toggleMaximize();

  // macOS convention puts the traffic lights on the left; Windows/Linux put
  // them on the right.
  const isMac = $derived(
    typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.userAgent),
  );

  function onBarPointerDown(e: PointerEvent) {
    if (e.button !== 0) return;
    const t = e.target as HTMLElement | null;
    if (t?.closest("button, input, select, a")) return;
    e.preventDefault();
    getCurrentWindow().startDragging();
  }

  function onBarDoubleClick(e: MouseEvent) {
    const t = e.target as HTMLElement | null;
    if (t?.closest("button, input, select, a")) return;
    getCurrentWindow().toggleMaximize();
  }

  onMount(async () => {
    loadAndApplyTheme();
    try {
      const detected = await invoke<InstallerStatus>("installer_status");
      info = detected;
      targetDir = detected.default_install_dir;

      const requestedMode = page.url.searchParams.get("mode");
      if (requestedMode === "uninstall" || requestedMode === "maintenance") {
        mode = "maintenance";
      } else if (requestedMode === "install") {
        mode = "install";
      } else {
        mode = detected.is_installed ? "maintenance" : "install";
      }
    } catch {
      targetDir = "C:\\Program Files\\LewdZone";
    } finally {
      statusLoading = false;
    }
  });

  async function startInstallation() {
    installTab = "installing";
    inProgress = true;
    progressPct = 15;
    progressStep = "Validating target directory…";

    try {
      await new Promise((r) => setTimeout(r, 400));
      progressPct = 40;
      progressStep = "Copying program binaries & resources…";

      const res = await invoke<OperationResult>("installer_install", {
        options: {
          target_dir: targetDir,
          create_desktop_shortcut: createDesktop,
          create_start_menu_shortcut: createStartMenu,
          add_to_path: addToPath,
          launch_after: launchAfter,
        },
      });

      progressPct = 85;
      progressStep = "Configuring system shortcuts & associations…";
      await new Promise((r) => setTimeout(r, 300));

      progressPct = 100;
      progressStep = "Installation complete!";
      result = res;
      logDetails = res.details;
      installTab = "complete";
    } catch (err) {
      result = {
        success: false,
        message: String(err),
        details: [String(err)],
      };
      installTab = "complete";
    } finally {
      inProgress = false;
    }
  }

  async function onFinish() {
    if (launchAfter && result?.success) {
      try {
        await invoke("installer_launch_app", { targetDir: targetDir || null });
      } catch (e) {
        console.warn("Failed to launch app after install:", e);
      }
    }
    closeWindow();
  }

  async function startUninstallation() {
    maintenanceTab = "removing";
    inProgress = true;
    progressPct = 25;
    progressStep = "Terminating running instances…";

    try {
      await new Promise((r) => setTimeout(r, 400));
      progressPct = 60;
      progressStep = "Removing shortcuts and program files…";

      const res = await invoke<OperationResult>("installer_uninstall", {
        options: {
          remove_user_data: removeUserData,
        },
      });

      progressPct = 100;
      progressStep = "Removal complete!";
      result = res;
      logDetails = res.details;
      maintenanceTab = "finished";
    } catch (err) {
      result = {
        success: false,
        message: String(err),
        details: [String(err)],
      };
      maintenanceTab = "finished";
    } finally {
      inProgress = false;
    }
  }

  function formatBytes(bytes: number): string {
    const gb = bytes / (1024 * 1024 * 1024);
    return `${gb.toFixed(1)} GB`;
  }
</script>

<div class="installer-shell">
  <!-- Custom frameless titlebar matching the main app -->
  <header
    class="titlebar"
    class:mac={isMac}
    role="presentation"
    onpointerdown={onBarPointerDown}
    ondblclick={onBarDoubleClick}
  >
    <div class="traffic" aria-label="Window controls">
      <button class="dot close" aria-label="Close window" onclick={closeWindow}></button>
      <button class="dot min" aria-label="Minimize window" onclick={minimizeWindow}></button>
      <button class="dot max" aria-label="Maximize window" onclick={maximizeWindow}></button>
    </div>
    <div class="app-badge">
      <svg class="app-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="6.5" width="18" height="11" rx="5.5"/>
        <circle cx="8" cy="11.5" r="1.1" fill="currentColor"/>
        <circle cx="12.5" cy="11.5" r="1.1" fill="currentColor"/>
      </svg>
      <span class="app-title">LewdZone Setup</span>
      {#if info}
        <span class="version-tag">v{info.current_version}</span>
      {/if}
    </div>
    <div class="spacer"></div>
  </header>

  {#if statusLoading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Initializing setup wizard…</p>
    </div>
  {:else if mode === "install"}
    <!-- ==================== INSTALLER WIZARD ==================== -->
    <div class="wizard-body">
      <!-- Modern Navigation Tabs -->
      <nav class="tab-strip" aria-label="Installer navigation">
        <button
          class="tab-btn"
          class:active={installTab === "welcome"}
          onclick={() => { if (!inProgress && installTab !== "complete") installTab = "welcome"; }}
        >
          <span class="tab-num">1</span>
          <span class="tab-text">Welcome</span>
        </button>
        <button
          class="tab-btn"
          class:active={installTab === "destination"}
          onclick={() => { if (!inProgress && installTab !== "complete") installTab = "destination"; }}
        >
          <span class="tab-num">2</span>
          <span class="tab-text">Location</span>
        </button>
        <button
          class="tab-btn"
          class:active={installTab === "options"}
          onclick={() => { if (!inProgress && installTab !== "complete") installTab = "options"; }}
        >
          <span class="tab-num">3</span>
          <span class="tab-text">Options</span>
        </button>
        <button
          class="tab-btn"
          class:active={installTab === "installing" || installTab === "complete"}
          disabled={installTab !== "installing" && installTab !== "complete"}
        >
          <span class="tab-num">4</span>
          <span class="tab-text">Install</span>
        </button>
      </nav>

      <!-- Tab Content Area -->
      <main class="wizard-page">
        {#if installTab === "welcome"}
          <div class="page-content hero-page">
            <div class="hero-left">
              <h2>Welcome to LewdZone</h2>
              <p class="hero-desc">
                The high-performance, cross-platform game launcher for lewdzone.com.
                Direct file streaming, instant metadata enrichment, and seamless one-click launches.
              </p>
              <div class="feature-list">
                <div class="feat-item">
                  <span class="feat-dot">✦</span>
                  <div>
                    <strong>Direct In-App Downloads:</strong> Stream and extract multi-gigabyte releases with live byte progress.
                  </div>
                </div>
                <div class="feat-item">
                  <span class="feat-dot">✦</span>
                  <div>
                    <strong>Metadata & Artwork:</strong> Enriched with SteamGridDB and IGDB for high-res cover art.
                  </div>
                </div>
                <div class="feat-item">
                  <span class="feat-dot">✦</span>
                  <div>
                    <strong>Cross-Platform Parity:</strong> Full feature support across Windows, macOS, and Linux.
                  </div>
                </div>
              </div>
            </div>
          </div>

        {:else if installTab === "destination"}
          <div class="page-content">
            <h3>Choose Install Location</h3>
            <p class="section-desc">Setup will install LewdZone Launcher into the following directory:</p>

            <div class="path-card">
              <label class="path-label" for="target-path">Destination Folder:</label>
              <div class="path-row">
                <input id="target-path" type="text" class="path-input" bind:value={targetDir} />
              </div>
            </div>

            <div class="space-info">
              <div class="space-row">
                <span>Space required:</span>
                <strong>~120 MB</strong>
              </div>
              <div class="space-row">
                <span>Space available:</span>
                <strong class="space-ok">Plenty available</strong>
              </div>
            </div>
          </div>

        {:else if installTab === "options"}
          <div class="page-content">
            <h3>Select Additional Tasks</h3>
            <p class="section-desc">Configure desktop integration and launch preferences:</p>

            <div class="options-group">
              <label class="check-option">
                <input type="checkbox" bind:checked={createDesktop} />
                <div class="opt-desc">
                  <strong>Create a Desktop Shortcut</strong>
                  <span>Place a quick-launch shortcut on your desktop</span>
                </div>
              </label>

              <label class="check-option">
                <input type="checkbox" bind:checked={createStartMenu} />
                <div class="opt-desc">
                  <strong>Create a Start Menu Shortcut</strong>
                  <span>Register in your application menu for quick search</span>
                </div>
              </label>

              <label class="check-option">
                <input type="checkbox" bind:checked={addToPath} />
                <div class="opt-desc">
                  <strong>Enable CLI Access (PATH)</strong>
                  <span>Allows running the <code>lewdzone</code> command in terminal</span>
                </div>
              </label>

              <label class="check-option">
                <input type="checkbox" bind:checked={launchAfter} />
                <div class="opt-desc">
                  <strong>Launch after finish</strong>
                  <span>Open LewdZone Launcher immediately after setup completes</span>
                </div>
              </label>
            </div>
          </div>

        {:else if installTab === "installing"}
          <div class="page-content progress-page">
            <h3>Installing LewdZone Launcher…</h3>
            <p class="section-desc">{progressStep}</p>

            <div class="progress-bar-bg">
              <div class="progress-bar-fill" style="width: {progressPct}%"></div>
            </div>

            <div class="log-toggle-row">
              <button class="link-btn" onclick={() => (showLogs = !showLogs)}>
                {showLogs ? "Hide details" : "Show details"}
              </button>
            </div>

            {#if showLogs}
              <div class="install-logs">
                {#each logDetails as item}
                  <div>{item}</div>
                {/each}
              </div>
            {/if}
          </div>

        {:else if installTab === "complete"}
          <div class="page-content complete-page">
            <div class="success-icon">✓</div>
            <h3>Installation Completed</h3>
            <p class="complete-desc">
              {result?.message ?? "LewdZone Launcher has been successfully installed on your computer."}
            </p>
            <p class="launch-hint">Click Finish to exit setup and begin exploring your catalog.</p>
          </div>
        {/if}
      </main>

      <!-- Wizard Action Buttons -->
      <footer class="wizard-footer">
        {#if installTab === "welcome"}
          <button class="btn secondary" onclick={closeWindow}>Cancel</button>
          <button class="btn primary" onclick={() => (installTab = "destination")}>Next ›</button>
        {:else if installTab === "destination"}
          <button class="btn secondary" onclick={() => (installTab = "welcome")}>‹ Back</button>
          <button class="btn primary" onclick={() => (installTab = "options")}>Next ›</button>
        {:else if installTab === "options"}
          <button class="btn secondary" onclick={() => (installTab = "destination")}>‹ Back</button>
          <button class="btn primary install" onclick={startInstallation}>Install</button>
        {:else if installTab === "installing"}
          <div class="spacer"></div>
          <button class="btn" disabled>Installing…</button>
        {:else if installTab === "complete"}
          <div class="spacer"></div>
          <button class="btn primary" onclick={onFinish}>Finish</button>
        {/if}
      </footer>
    </div>

  {:else}
    <!-- ==================== MAINTENANCE & UNINSTALLER WIZARD ==================== -->
    <div class="wizard-body">
      <nav class="tab-strip" aria-label="Maintenance navigation">
        <button
          class="tab-btn"
          class:active={maintenanceTab === "manage"}
          onclick={() => { if (!inProgress && maintenanceTab !== "finished") maintenanceTab = "manage"; }}
        >
          <span class="tab-num">1</span>
          <span class="tab-text">Maintenance</span>
        </button>
        <button
          class="tab-btn"
          class:active={maintenanceTab === "uninstall_options"}
          onclick={() => { if (!inProgress && maintenanceTab !== "finished") maintenanceTab = "uninstall_options"; }}
        >
          <span class="tab-num">2</span>
          <span class="tab-text">Options</span>
        </button>
        <button
          class="tab-btn"
          class:active={maintenanceTab === "removing" || maintenanceTab === "finished"}
          disabled={maintenanceTab !== "removing" && maintenanceTab !== "finished"}
        >
          <span class="tab-num">3</span>
          <span class="tab-text">Complete</span>
        </button>
      </nav>

      <main class="wizard-page">
        {#if maintenanceTab === "manage"}
          <div class="page-content">
            <h3>Manage Installation</h3>
            <p class="section-desc">
              LewdZone Launcher is currently installed at:
              <code>{info?.default_install_dir}</code>
            </p>

            <div class="maintenance-cards">
              <button class="card-btn" onclick={() => (mode = "install", installTab = "destination")}>
                <div class="card-title">Reinstall / Update</div>
                <div class="card-sub">Reinstall or upgrade the launcher to version {info?.current_version}</div>
              </button>

              <button class="card-btn" onclick={startInstallation}>
                <div class="card-title">Repair Shortcuts</div>
                <div class="card-sub">Recreate missing Desktop and Start Menu application links</div>
              </button>

              <button class="card-btn danger" onclick={() => (maintenanceTab = "uninstall_options")}>
                <div class="card-title">Uninstall LewdZone</div>
                <div class="card-sub">Remove the application and registered system handlers</div>
              </button>
            </div>
          </div>

        {:else if maintenanceTab === "uninstall_options"}
          <div class="page-content">
            <h3>Uninstall Options</h3>
            <p class="section-desc">Choose how user data and games are handled:</p>

            <div class="options-group">
              <label class="check-option">
                <input type="checkbox" bind:checked={removeUserData} />
                <div class="opt-desc">
                  <strong>Delete all library and configuration data</strong>
                  <span>Removes cached artwork, SQLite database, and configuration settings.</span>
                </div>
              </label>
            </div>

            <div class="warn-box">
              Downloaded games in external extraction folders will remain safe and untouched.
            </div>
          </div>

        {:else if maintenanceTab === "removing"}
          <div class="page-content progress-page">
            <h3>Uninstalling LewdZone Launcher…</h3>
            <p class="section-desc">{progressStep}</p>

            <div class="progress-bar-bg">
              <div class="progress-bar-fill danger" style="width: {progressPct}%"></div>
            </div>
          </div>

        {:else if maintenanceTab === "finished"}
          <div class="page-content complete-page">
            <div class="success-icon">✓</div>
            <h3>Uninstallation Completed</h3>
            <p class="complete-desc">
              {result?.message ?? "LewdZone Launcher has been successfully removed from your computer."}
            </p>
          </div>
        {/if}
      </main>

      <footer class="wizard-footer">
        {#if maintenanceTab === "manage"}
          <button class="btn secondary" onclick={closeWindow}>Cancel</button>
        {:else if maintenanceTab === "uninstall_options"}
          <button class="btn secondary" onclick={() => (maintenanceTab = "manage")}>‹ Back</button>
          <button class="btn danger" onclick={startUninstallation}>Uninstall Now</button>
        {:else if maintenanceTab === "removing"}
          <div class="spacer"></div>
          <button class="btn" disabled>Removing…</button>
        {:else if maintenanceTab === "finished"}
          <div class="spacer"></div>
          <button class="btn primary" onclick={closeWindow}>Close</button>
        {/if}
      </footer>
    </div>
  {/if}
</div>

<style>
  :global(body) {
    margin: 0;
    user-select: none;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
  }

  .installer-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #0d1117;
    color: #e6edf3;
    overflow: hidden;
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
    background: var(--lz-glass, rgba(22, 27, 34, 0.85));
    backdrop-filter: blur(10px);
    border-bottom: 1px solid var(--lz-surface-2, #30363d);
    user-select: none;
    cursor: grab;
  }

  /* Default (Windows/Linux): traffic lights on the right. */
  .titlebar .app-badge {
    order: 1;
  }

  .titlebar .spacer {
    order: 2;
  }

  .titlebar .traffic {
    order: 3;
  }

  /* macOS: traffic lights on the left, before the badge. */
  .titlebar.mac .traffic {
    order: 0;
    margin-right: 2px;
  }

  .app-badge {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .app-icon {
    width: 18px;
    height: 18px;
    color: var(--lz-cyan, #58a6ff);
  }

  .app-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--lz-text, #e6edf3);
  }

  .version-tag {
    font-size: 11px;
    color: var(--lz-text-dim, #8b949e);
    background: var(--lz-surface-2, #21262d);
    padding: 2px 6px;
    border-radius: 4px;
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

  .spacer {
    flex: 1 1 auto;
  }

  .wizard-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .tab-strip {
    height: 48px;
    background: #161b22;
    border-bottom: 1px solid #30363d;
    display: flex;
    padding: 0 16px;
    gap: 8px;
  }

  .tab-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: #8b949e;
    font-size: 13px;
    font-weight: 500;
    padding: 0 12px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .tab-btn.active {
    color: #58a6ff;
    border-bottom-color: #58a6ff;
  }

  .tab-num {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #21262d;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    font-weight: 700;
  }

  .tab-btn.active .tab-num {
    background: #58a6ff;
    color: #0d1117;
  }

  .wizard-page {
    flex: 1;
    padding: 24px 32px;
    overflow-y: auto;
  }

  .page-content h2,
  .page-content h3 {
    margin: 0 0 8px;
    font-weight: 600;
  }

  .section-desc {
    color: #8b949e;
    font-size: 13px;
    margin: 0 0 20px;
  }

  .hero-page {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .hero-desc {
    color: #8b949e;
    font-size: 14px;
    line-height: 1.5;
    margin: 0 0 12px;
  }

  .feature-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 8px;
    padding: 16px;
  }

  .feat-item {
    display: flex;
    gap: 10px;
    font-size: 13px;
    line-height: 1.4;
  }

  .feat-dot {
    color: #58a6ff;
  }

  .path-card {
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 8px;
    padding: 14px;
    margin-bottom: 16px;
  }

  .path-label {
    display: block;
    font-size: 12px;
    font-weight: 600;
    color: #8b949e;
    margin-bottom: 6px;
  }

  .path-row {
    display: flex;
    gap: 8px;
  }

  .path-input {
    flex: 1;
    background: #0d1117;
    border: 1px solid #30363d;
    color: #e6edf3;
    padding: 8px 12px;
    border-radius: 6px;
    font-size: 13px;
  }

  .space-info {
    font-size: 12px;
    color: #8b949e;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .space-row {
    display: flex;
    justify-content: space-between;
  }

  .space-ok {
    color: #3fb950;
  }

  .options-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .check-option {
    display: flex;
    gap: 12px;
    align-items: flex-start;
    padding: 10px 14px;
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 6px;
    cursor: pointer;
  }

  .check-option input {
    margin-top: 3px;
  }

  .opt-desc {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 13px;
  }

  .opt-desc span {
    font-size: 11px;
    color: #8b949e;
  }

  .progress-page {
    display: flex;
    flex-direction: column;
    justify-content: center;
  }

  .progress-bar-bg {
    height: 10px;
    background: #21262d;
    border-radius: 5px;
    overflow: hidden;
    margin-bottom: 12px;
  }

  .progress-bar-fill {
    height: 100%;
    background: #238636;
    transition: width 0.3s ease;
  }

  .progress-bar-fill.danger {
    background: #da3633;
  }

  .log-toggle-row {
    display: flex;
    justify-content: flex-end;
  }

  .link-btn {
    background: transparent;
    border: none;
    color: #58a6ff;
    font-size: 12px;
    cursor: pointer;
    text-decoration: underline;
  }

  .install-logs {
    margin-top: 12px;
    background: #0d1117;
    border: 1px solid #30363d;
    border-radius: 6px;
    padding: 8px 12px;
    max-height: 120px;
    overflow-y: auto;
    font-family: monospace;
    font-size: 11px;
    color: #8b949e;
    line-height: 1.4;
  }

  .complete-page {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding-top: 24px;
  }

  .success-icon {
    width: 52px;
    height: 52px;
    border-radius: 50%;
    background: rgba(63, 185, 80, 0.15);
    color: #3fb950;
    font-size: 26px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 16px;
    border: 2px solid #3fb950;
  }

  .complete-desc {
    color: #8b949e;
    font-size: 14px;
    max-width: 480px;
    margin: 0 0 8px;
  }

  .launch-hint {
    color: #58a6ff;
    font-size: 12px;
    margin: 0;
  }

  .maintenance-cards {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-top: 16px;
  }

  .card-btn {
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 8px;
    padding: 16px;
    text-align: left;
    color: #e6edf3;
    cursor: pointer;
    transition: all 0.15s;
  }

  .card-btn:hover {
    border-color: #58a6ff;
    background: #1c2128;
  }

  .card-btn.danger:hover {
    border-color: #da3633;
    background: rgba(218, 54, 51, 0.1);
  }

  .card-title {
    font-weight: 600;
    font-size: 14px;
    margin-bottom: 4px;
  }

  .card-sub {
    font-size: 12px;
    color: #8b949e;
  }

  .warn-box {
    margin-top: 16px;
    padding: 12px;
    border-radius: 6px;
    background: rgba(210, 153, 34, 0.1);
    border: 1px solid rgba(210, 153, 34, 0.4);
    font-size: 12px;
    color: #d29922;
  }

  .wizard-footer {
    height: 56px;
    background: #161b22;
    border-top: 1px solid #30363d;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: 0 24px;
    gap: 12px;
  }

  .btn {
    padding: 8px 18px;
    font-size: 13px;
    font-weight: 600;
    border-radius: 6px;
    cursor: pointer;
    border: 1px solid transparent;
  }

  .btn.primary {
    background: #238636;
    color: #fff;
  }

  .btn.primary:hover:not(:disabled) {
    background: #2ea043;
  }

  .btn.primary.install {
    background: #1f6feb;
  }

  .btn.primary.install:hover:not(:disabled) {
    background: #388bfd;
  }

  .btn.secondary {
    background: #21262d;
    color: #c9d1d9;
    border-color: #30363d;
  }

  .btn.secondary:hover:not(:disabled) {
    background: #30363d;
  }

  .btn.danger {
    background: #da3633;
    color: #fff;
  }

  .btn.danger:hover:not(:disabled) {
    background: #f85149;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .spacer {
    flex: 1;
  }

  .loading-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: #8b949e;
    font-size: 13px;
  }

  .spinner {
    width: 28px;
    height: 28px;
    border: 3px solid #30363d;
    border-top-color: #58a6ff;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
