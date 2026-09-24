import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import InstallerPage from "../../routes/installer/+page.svelte";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("$app/state", () => ({
  page: { url: new URL("http://localhost/installer") },
}));

vi.mock("@tauri-apps/api/window", () => {
  const win = {
    minimize: vi.fn(),
    close: vi.fn(),
    startDragging: vi.fn(),
  };
  return { getCurrentWindow: vi.fn(() => win) };
});

describe("unified installer / uninstaller wizard", () => {
  const mockStatus = {
    is_installed: false,
    installed_version: null,
    current_version: "0.2.1",
    default_install_dir: "C:\\Users\\Test\\AppData\\Local\\Programs\\LewdZone",
    existing_binary: null,
    os: "windows",
  };

  const mockDiskSpace = {
    available_bytes: 50 * 1024 * 1024 * 1024,
    required_bytes: 250 * 1024 * 1024,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockImplementation(async (cmd: string, args?: any) => {
      if (cmd === "installer_status") {
        return mockStatus;
      }
      if (cmd === "installer_disk_space") {
        return mockDiskSpace;
      }
      if (cmd === "installer_install") {
        return {
          success: true,
          message: "Installed successfully",
          details: ["Copied binary", "Created shortcut"],
        };
      }
      if (cmd === "installer_uninstall") {
        return {
          success: true,
          message: "Uninstalled successfully",
          details: ["Removed files"],
        };
      }
      return null;
    });
  });

  it("renders welcome screen with version and allows navigation through install steps", async () => {
    const user = userEvent.setup();
    render(InstallerPage);

    // Welcome step
    expect(await screen.findByText("Welcome to LewdZone")).toBeTruthy();
    expect(screen.getByText("v0.2.1")).toBeTruthy();

    // Next -> Location step
    const nextBtn = screen.getByRole("button", { name: /Next/i });
    await user.click(nextBtn);

    expect(await screen.findByText("Choose Install Location")).toBeTruthy();
    expect(screen.getByText(/Space required:/i)).toBeTruthy();

    // Next -> Options step
    await user.click(screen.getByRole("button", { name: /Next/i }));
    expect(await screen.findByText("Select Additional Tasks")).toBeTruthy();
    expect(screen.getByText("Create a Desktop Shortcut")).toBeTruthy();

    // Back button works
    await user.click(screen.getByRole("button", { name: /Back/i }));
    expect(await screen.findByText("Choose Install Location")).toBeTruthy();
  });

  it("switches to maintenance mode if already installed", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "installer_status") {
        return {
          ...mockStatus,
          is_installed: true,
          installed_version: "0.2.1",
          existing_binary: "C:\\Users\\Test\\AppData\\Local\\Programs\\LewdZone\\lewdzone.exe",
        };
      }
      return null;
    });

    render(InstallerPage);

    expect(await screen.findByText("Manage Installation")).toBeTruthy();
    expect(screen.getByText(/LewdZone Launcher is currently installed at/i)).toBeTruthy();
    expect(screen.getByText("Uninstall LewdZone")).toBeTruthy();
    expect(screen.getByText("Repair Shortcuts")).toBeTruthy();
  });
});
