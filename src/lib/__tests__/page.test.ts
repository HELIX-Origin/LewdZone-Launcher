import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async () => "Hello, test! from Rust"),
}));

vi.mock("$app/state", () => ({
  page: { url: new URL("http://localhost/store") },
}));

vi.mock("$app/navigation", () => ({
  goto: vi.fn(),
}));

vi.mock("@tauri-apps/api/window", () => {
  const win = {
    minimize: vi.fn(),
    toggleMaximize: vi.fn(),
    close: vi.fn(),
    startDragging: vi.fn(),
  };
  return { getCurrentWindow: vi.fn(() => win) };
});

import Layout from "../../routes/+layout.svelte";
import SettingsPage from "../../routes/settings/+page.svelte";
import StorePage from "../../routes/store/+page.svelte";
import GamePage from "../../routes/store/[slug]/+page.svelte";
import LibraryPage from "../../routes/library/+page.svelte";
import FavoritesPage from "../../routes/favorites/+page.svelte";
import DownloadsPage from "../../routes/downloads/+page.svelte";

const sampleGame = {
  slug: "wild-life",
  title: "Wild Life",
  thumb_url: "https://h1.lzcdn.com/img/wild-life-cover.jpg",
  platforms: ["pc"],
  engine: "Unreal Engine",
  state: "Ongoing",
  version_tag: "v2026-06-15 Full",
  developer: "Adeptus Steve",
  genre_slugs: ["3d-games"],
  genres: ["3D Game"],
  external_genres: [],
  views: "962K",
};

const sampleGenres = [
  { label: "3D Game", slug: "3d-games", count: 1740 },
  { label: "Adventure", slug: "adventure", count: 3314 },
];

const sampleGameData = {
  slug: "wild-life",
  post_id: 54321,
  title: "Wild Life",
  developer: "Adeptus Steve",
  current_version: "v2026-06-15 Full",
  engine: "Unreal Engine",
  platforms: ["pc"],
  genres: ["3d-games", "adventure"],
  external_genres: [],
  rating: null,
  size_label: "5.0 GB",
  censorship: "Uncensored",
  screenshots: ["https://h1.lzcdn.com/img/wild-life-shot.jpg"],
  description: "A mad universe of choices.",
  versions: [
    {
      label: "v2026-06-15 Full",
      is_latest: true,
      official: [
        {
          label: "FULL",
          variant: null,
          host: "fileknot",
          platform: "pc",
          go_link: "https://lewdzone.com/go/#t=v1.a.b",
        },
      ],
      community: [],
    },
  ],
  download_entries: [
    {
      label: "FULL",
      variant: null,
      host: "fileknot",
      platform: "pc",
      go_link: "https://lewdzone.com/go/#t=v1.a.b",
    },
  ],
};

const sampleJob = {
  id: 1,
  slug: "wild-life",
  version: "v2026-06-15 Full",
  platform: "PC",
  tab: "official",
  source: "fileknot",
  status: "queued",
  message: null,
  bytes_done: 0,
  bytes_total: 0,
  created_at: 1_716_000_000,
  updated_at: 1_716_000_000,
};

const sampleSources = [
  { host: "fileknot", label: "native cloud app", preferred: false },
];

type InvokeArgs = Record<string, unknown>;

function mockInvoke(impl: (cmd: string, args?: InvokeArgs) => unknown) {
  const invokeMock = invoke as unknown as ReturnType<typeof vi.fn>;
  invokeMock.mockReset();
  invokeMock.mockImplementation(impl);
  return invokeMock;
}

describe("app shell", () => {
  it("renders the left icon sidebar with primary tabs", () => {
    render(Layout);
    for (const label of ["Favorites", "Library", "Store", "Queue", "Settings"]) {
      expect(screen.getByRole("button", { name: label })).toBeInTheDocument();
    }
    expect(screen.queryByRole("button", { name: "Home" })).not.toBeInTheDocument();
    const sidebar = screen.getByRole("complementary", { name: "Primary" });
    expect(sidebar).toBeInTheDocument();
  });

  it("renders the custom macOS-style title bar with traffic light controls", () => {
    render(Layout);
    expect(screen.getByText("LewdZone Launcher")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Close window" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Minimize window" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Maximize window" }),
    ).toBeInTheDocument();
  });

  it("renders a custom menu bar with items without profile button", () => {
    render(Layout);
    for (const label of ["File", "View", "Help"]) {
      expect(screen.getByRole("button", { name: label })).toBeInTheDocument();
    }
    expect(
      screen.queryByRole("button", { name: "Profile" }),
    ).not.toBeInTheDocument();
  });

  it("traffic-light buttons call the window controls, not the drag", async () => {
    const user = userEvent.setup();
    render(Layout);
    await user.click(screen.getByRole("button", { name: "Minimize window" }));
    await user.click(screen.getByRole("button", { name: "Maximize window" }));
    await user.click(screen.getByRole("button", { name: "Close window" }));
    const win = getCurrentWindow();
    expect(win.minimize).toHaveBeenCalledTimes(1);
    expect(win.toggleMaximize).toHaveBeenCalledTimes(1);
    expect(win.close).toHaveBeenCalledTimes(1);
    expect(win.startDragging).not.toHaveBeenCalled();
  });

  it("opens the Help → About dialog from the menu bar", async () => {
    const user = userEvent.setup();
    render(Layout);
    await user.click(screen.getByRole("button", { name: "Help" }));
    await user.click(
      screen.getByRole("menuitem", { name: "About LewdZone Launcher" }),
    );
    const dialog = screen.getByRole("dialog", { name: "About" });
    expect(dialog).toBeInTheDocument();
    expect(screen.getAllByText("LewdZone Launcher").length).toBeGreaterThan(0);
  });
});

describe("store page", () => {
  beforeEach(() => {
    mockInvoke(async (cmd: string, args?: InvokeArgs) => {
      if (cmd === "catalog_page") {
        return {
          games: [sampleGame],
          meta: { page: args?.page ?? 1, total_pages: 787 },
        };
      }
      if (cmd === "catalog_genres") {
        return sampleGenres;
      }
      return [];
    });
  });

  it("renders catalog tiles fetched from Rust", async () => {
    render(StorePage);
    expect(await screen.findByText("Wild Life")).toBeInTheDocument();
    expect(await screen.findByText("1 games · page 1 of 787")).toBeInTheDocument();
  });

  it("renders the catalog grid role", async () => {
    render(StorePage);
    expect(await screen.findByText("Wild Life")).toBeInTheDocument();
    expect(screen.getAllByRole("listitem").length).toBeGreaterThan(0);
  });

  it("sends the full filter object with the page", async () => {
    render(StorePage);
    await screen.findByText("Wild Life");
    const invokeMock = invoke as unknown as ReturnType<typeof vi.fn>;
    const call = invokeMock.mock.calls.find((c) => c[0] === "catalog_page");
    expect(call).toBeTruthy();
    expect(call?.[1]).toEqual(
      expect.objectContaining({
        page: 1,
        filter: expect.objectContaining({ sort: "Popularity" }),
      }),
    );
  });

  it("loads the genre cloud into the category rail", async () => {
    render(StorePage);
    expect(await screen.findByText("Adventure")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Include 3D Game" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Exclude 3D Game" })).toBeInTheDocument();
  });
});

describe("store game detail page", () => {
  beforeEach(() => {
    mockInvoke(async (cmd: string) => {
      if (cmd === "game_page") return sampleGameData;
      if (cmd === "game_sources") return sampleSources;
      if (cmd === "game_download") return sampleJob;
      if (cmd === "open_resolver_window") return;
      if (cmd === "content_enrich") {
        return { description: null, developer: null, rating: null, tags: [], screenshots: [], genres: [] };
      }
      return [];
    });
  });

  it("renders the loaded game's meta and download panel", async () => {
    render(GamePage);
    expect(await screen.findByText("Wild Life")).toBeInTheDocument();
    expect(screen.getByText("by Adeptus Steve")).toBeInTheDocument();
    expect(screen.getByText("5.0 GB")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Download via Fileknot" })).toBeInTheDocument();
  });

  it("filters out unsupported hosts and keeps supported hosts", async () => {
    const gameWithBoth = {
      ...sampleGameData,
      versions: [
        {
          label: "v1.0",
          is_latest: true,
          official: [
            {
              label: "Gofile Link",
              variant: null,
              host: "gofile",
              platform: "pc",
              go_link: "https://lewdzone.com/go/#t=v1.gofile",
            },
            {
              label: "Mega Link",
              variant: null,
              host: "mega",
              platform: "pc",
              go_link: "https://lewdzone.com/go/#t=v1.mega",
            },
          ],
          community: [],
        },
      ],
    };
    mockInvoke(async (cmd: string) => {
      if (cmd === "game_page") return gameWithBoth;
      return [];
    });
    render(GamePage);
    await screen.findByText("Wild Life");
    expect(screen.getByRole("button", { name: "Download via Mega" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /gofile/i })).not.toBeInTheDocument();
  });

  it("opens the secure resolver window when a source button is clicked", async () => {
    const user = userEvent.setup();
    render(GamePage);
    const btn = await screen.findByRole("button", { name: "Download via Fileknot" });
    await user.click(btn);
    expect(invoke).toHaveBeenCalledWith(
      "open_resolver_window",
      expect.objectContaining({
        slug: "wild-life",
        host: "fileknot",
        url: "https://lewdzone.com/go/#t=v1.a.b",
      }),
    );
  });
});

describe("library page", () => {
  it("shows the empty state when nothing is installed", async () => {
    mockInvoke(async (cmd: string) => {
      if (cmd === "library_list") return { root: "/fake/library", games: [] };
      if (cmd === "favorites_list") return [];
      return [];
    });
    render(LibraryPage);
    expect(await screen.findByRole("heading", { name: "Library" })).toBeInTheDocument();
    expect(screen.getByText(/Nothing installed yet/)).toBeInTheDocument();
  });

  it("lists installed games from manifests", async () => {
    mockInvoke(async (cmd: string) => {
      if (cmd === "library_list") {
        return {
          root: "/fake/library",
          games: [
            {
              slug: "wild-life",
              post_id: 54321,
              title: "Wild Life",
              version: "v2026",
              platform: "pc",
              tab: "fileknot",
              engine: "Unity",
              install_path: "/fake/library/lzapps/wild-life",
              candidates: ["WildLife.exe"],
              launch_exe: "",
              installed_at: "2026-01-01T00:00:00Z",
              size_on_disk: 5_368_709_120,
            },
            {
              slug: "treasure-of-nadia",
              post_id: 111,
              title: "Treasure of Nadia",
              version: "1.0117",
              platform: "pc",
              tab: "fileknot",
              engine: "Ren'Py",
              install_path: "/fake/library/lzapps/treasure-of-nadia",
              candidates: ["TreasureOfNadia.exe"],
              launch_exe: "",
              installed_at: "2026-01-02T00:00:00Z",
              size_on_disk: 2_147_483_648,
            },
          ],
        };
      }
      if (cmd === "favorites_list") return [];
      return [];
    });
    render(LibraryPage);
    expect(await screen.findByText("Wild Life")).toBeInTheDocument();
    expect(screen.getByText("Treasure of Nadia")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Launch Wild Life/ })).toBeInTheDocument();
  });

  it("invokes game_launch when the Launch button is clicked", async () => {
    const user = userEvent.setup();
    const invocations: Array<{ cmd: string; args?: InvokeArgs }> = [];
    mockInvoke(async (cmd: string, args?: InvokeArgs) => {
      invocations.push({ cmd, args });
      if (cmd === "library_list") {
        return {
          root: "/fake/library",
          games: [
            {
              slug: "wild-life",
              post_id: 54321,
              title: "Wild Life",
              version: "v2026",
              platform: "pc",
              tab: "fileknot",
              engine: "Unity",
              install_path: "/fake/library/lzapps/wild-life",
              candidates: ["WildLife.exe"],
              launch_exe: "",
              installed_at: "2026-01-01T00:00:00Z",
              size_on_disk: 5_368_709_120,
            },
          ],
        };
      }
      if (cmd === "favorites_list") return [];
      return [];
    });
    render(LibraryPage);
    const btn = await screen.findByRole("button", { name: /Launch Wild Life/ });
    await user.click(btn);
    expect(invocations.some((i) => i.cmd === "game_launch" && i.args?.slug === "wild-life")).toBe(true);
  });

  it("shows a heart button on each installed game", async () => {
    mockInvoke(async (cmd: string) => {
      if (cmd === "library_list") {
        return {
          root: "/fake/library",
          games: [
            {
              slug: "wild-life",
              post_id: 54321,
              title: "Wild Life",
              version: "v2026",
              platform: "pc",
              tab: "fileknot",
              engine: "Unity",
              install_path: "/fake/library/lzapps/wild-life",
              candidates: ["WildLife.exe"],
              launch_exe: "",
              installed_at: "2026-01-01T00:00:00Z",
              size_on_disk: 5_368_709_120,
            },
          ],
        };
      }
      if (cmd === "favorites_list") return [];
      return [];
    });
    render(LibraryPage);
    expect(await screen.findByRole("button", { name: /Add Wild Life to favorites/ })).toBeInTheDocument();
  });

  it("invokes favorite_add when the heart button is clicked", async () => {
    const user = userEvent.setup();
    const invocations: Array<{ cmd: string; args?: InvokeArgs }> = [];
    mockInvoke(async (cmd: string, args?: InvokeArgs) => {
      invocations.push({ cmd, args });
      if (cmd === "library_list") {
        return {
          root: "/fake/library",
          games: [
            {
              slug: "wild-life",
              post_id: 54321,
              title: "Wild Life",
              version: "v2026",
              platform: "pc",
              tab: "fileknot",
              engine: "Unity",
              install_path: "/fake/library/lzapps/wild-life",
              candidates: ["WildLife.exe"],
              launch_exe: "",
              installed_at: "2026-01-01T00:00:00Z",
              size_on_disk: 5_368_709_120,
            },
          ],
        };
      }
      if (cmd === "favorites_list") return [];
      if (cmd === "favorite_add") return null;
      return [];
    });
    render(LibraryPage);
    const btn = await screen.findByRole("button", { name: /Add Wild Life to favorites/ });
    await user.click(btn);
    expect(invocations.some((i) => i.cmd === "favorite_add" && i.args?.slug === "wild-life")).toBe(true);
  });

  it("renders a desktop shortcut button and invokes create_shortcut", async () => {
    const user = userEvent.setup();
    const invocations: Array<{ cmd: string; args?: InvokeArgs }> = [];
    mockInvoke(async (cmd: string, args?: InvokeArgs) => {
      invocations.push({ cmd, args });
      if (cmd === "library_list") {
        return {
          root: "/fake/library",
          games: [
            {
              slug: "wild-life",
              post_id: 54321,
              title: "Wild Life",
              version: "v2026",
              platform: "pc",
              tab: "fileknot",
              engine: "Unity",
              install_path: "/fake/library/lzapps/wild-life",
              candidates: ["WildLife.exe"],
              launch_exe: "",
              installed_at: "2026-01-01T00:00:00Z",
              size_on_disk: 5_368_709_120,
            },
          ],
        };
      }
      if (cmd === "favorites_list") return [];
      if (cmd === "create_shortcut") return "/fake/Desktop/Wild Life.lnk";
      return [];
    });
    render(LibraryPage);
    const shortcutBtn = await screen.findByRole("button", { name: "Create shortcut for Wild Life" });
    expect(shortcutBtn).toBeInTheDocument();
    await user.click(shortcutBtn);
    expect(invocations.some((i) => i.cmd === "create_shortcut" && i.args?.slug === "wild-life")).toBe(true);
  });
});

describe("favorites page", () => {
  it("shows the empty state when there are no favorites", async () => {
    mockInvoke(async (cmd: string) => {
      if (cmd === "favorites_list") return [];
      return [];
    });
    render(FavoritesPage);
    expect(await screen.findByRole("heading", { name: "Favorites" })).toBeInTheDocument();
    expect(screen.getByText(/No favorites yet/)).toBeInTheDocument();
  });

  it("lists favorited games", async () => {
    mockInvoke(async (cmd: string) => {
      if (cmd === "favorites_list") {
        return [
          {
            slug: "wild-life",
            post_id: 54321,
            title: "Wild Life",
            thumb_url: null,
            platforms: ["pc"],
            engine: "Unity",
            state: "Ongoing",
            version_tag: "v2026",
            developer: "Adeptus Steve",
            genres: ["3D Game"],
            genre_slugs: ["3d-games"],
            external_genres: [],
            views: "962K",
          },
        ];
      }
      return [];
    });
    render(FavoritesPage);
    expect(await screen.findByText("Wild Life")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Remove Wild Life from favorites/ })).toBeInTheDocument();
  });

  it("invokes favorite_remove when the heart button is clicked", async () => {
    const user = userEvent.setup();
    const invocations: Array<{ cmd: string; args?: InvokeArgs }> = [];
    mockInvoke(async (cmd: string, args?: InvokeArgs) => {
      invocations.push({ cmd, args });
      if (cmd === "favorites_list") {
        return [
          {
            slug: "wild-life",
            post_id: 54321,
            title: "Wild Life",
            thumb_url: null,
            platforms: ["pc"],
            engine: "Unity",
            state: "Ongoing",
            version_tag: "v2026",
            developer: "Adeptus Steve",
            genres: ["3D Game"],
            genre_slugs: ["3d-games"],
            external_genres: [],
            views: "962K",
          },
        ];
      }
      return [];
    });
    render(FavoritesPage);
    const btn = await screen.findByRole("button", { name: /Remove Wild Life from favorites/ });
    await user.click(btn);
    expect(invocations.some((i) => i.cmd === "favorite_remove" && i.args?.slug === "wild-life")).toBe(true);
  });
});

describe("downloads page", () => {
  it("shows the empty state when the queue is empty", async () => {
    mockInvoke(async (cmd: string) => {
      if (cmd === "downloads_list") return [];
      return [];
    });
    render(DownloadsPage);
    expect(await screen.findByText("Downloads")).toBeInTheDocument();
    expect(
      screen.getByText(/No active downloads. Start one from the Store/),
    ).toBeInTheDocument();
  });

  it("lists queued, downloading, dispatched, and failed jobs from the queue", async () => {
    mockInvoke(async (cmd: string) => {
      if (cmd === "downloads_list") {
        return [
          sampleJob,
          {
            ...sampleJob,
            id: 2,
            status: "downloading",
            bytes_done: 2_147_483_648,
            bytes_total: 5_368_709_120,
            message: "downloaded 2.1 GB of 5 GB",
          },
          {
            ...sampleJob,
            id: 3,
            status: "dispatched",
            message: "downloaded 1 of 1 bytes",
          },
          {
            ...sampleJob,
            id: 4,
            status: "failed",
            message: "no matching download entries found",
          },
        ];
      }
      return [];
    });
    render(DownloadsPage);
    expect(await screen.findByText("Queued")).toBeInTheDocument();
    expect(screen.getByText("Downloading")).toBeInTheDocument();
    expect(screen.getByText("Dispatched")).toBeInTheDocument();
    expect(screen.getByText("Failed")).toBeInTheDocument();
    expect(screen.getAllByText("wild-life").length).toBe(4);
    expect(screen.getByText("downloaded 2.1 GB of 5 GB")).toBeInTheDocument();
    expect(screen.getByRole("progressbar", { name: /Download progress for wild-life/ })).toBeInTheDocument();
    expect(screen.getByText("no matching download entries found")).toBeInTheDocument();
  });

  it("allows cancelling or deleting active/queued jobs and deleting completed jobs", async () => {
    const user = userEvent.setup();
    const invocations: Array<{ cmd: string; args?: InvokeArgs }> = [];
    mockInvoke(async (cmd: string, args?: InvokeArgs) => {
      invocations.push({ cmd, args });
      if (cmd === "downloads_list") {
        return [
          sampleJob,
          { ...sampleJob, id: 3, status: "dispatched" },
        ];
      }
      return [];
    });
    render(DownloadsPage);
    const cancelBtn = await screen.findByRole("button", { name: "Cancel download for wild-life" });
    await user.click(cancelBtn);
    expect(invocations.some((i) => i.cmd === "download_cancel" && i.args?.id === 1)).toBe(true);

    const deleteBtns = screen.getAllByRole("button", { name: "Remove wild-life from downloads" });
    expect(deleteBtns.length).toBe(2);
    // Delete the dispatched job (id: 3)
    await user.click(deleteBtns[1]);
    expect(invocations.some((i) => i.cmd === "download_delete" && i.args?.id === 3)).toBe(true);
    // Delete the queued job directly (id: 1)
    await user.click(deleteBtns[0]);
    expect(invocations.some((i) => i.cmd === "download_delete" && i.args?.id === 1)).toBe(true);
  });

  it("renders extraction progress bar and badge for extracting jobs", async () => {
    mockInvoke(async (cmd: string) => {
      if (cmd === "downloads_list") {
        return [
          {
            ...sampleJob,
            id: 5,
            status: "extracting",
            bytes_done: 104_857_600,
            bytes_total: 209_715_200,
          },
        ];
      }
      return [];
    });
    render(DownloadsPage);
    expect(await screen.findByText("Extracting")).toBeInTheDocument();
    expect(screen.getByRole("progressbar", { name: /Extraction progress for wild-life/ })).toBeInTheDocument();
    expect(screen.getByText(/50% extracted/)).toBeInTheDocument();
  });
});

describe("settings page", () => {
  beforeEach(() => {
    mockInvoke(async (cmd: string, args?: InvokeArgs) => {
      switch (cmd) {
        case "settings_get":
          return {
            "library-root": "/fake/games",
            "capture-aware": false,
            "source-priority": "mega,google",
            theme: "",
            "sgdb-api-key": "(set)",
            "igdb-client-id": "(not set)",
            "igdb-client-secret": "(not set)",
          };
        case "themes_list":
          return ["midnight", "candy"];
        case "settings_set":
          return { key: args?.key, value: args?.value };
        default:
          return [];
      }
    });
  });

  it("loads snapshot + theme list into the page", async () => {
    render(SettingsPage);
    expect(await screen.findByText("Settings")).toBeInTheDocument();
    expect(await screen.findByDisplayValue("/fake/games")).toBeInTheDocument();
    expect(await screen.findByRole("option", { name: "midnight" })).toBeInTheDocument();
  });

  it("saves a text setting through settings_set", async () => {
    const user = userEvent.setup();
    render(SettingsPage);
    const list = await screen.findByDisplayValue("/fake/games");
    await user.type(list, "/SteamLibrary");
    await user.keyboard("{Enter}");

    expect(invoke).toHaveBeenCalledWith(
      "settings_set",
      expect.objectContaining({ key: "library-root", value: "/fake/games/SteamLibrary" }),
    );
    expect(await screen.findByText("library-root saved")).toBeInTheDocument();
  });

  it("does not render preferred download sources toggle controls", async () => {
    render(SettingsPage);
    await screen.findByDisplayValue("/fake/games");
    expect(screen.queryByText("Preferred download sources")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Toggle/ })).not.toBeInTheDocument();
  });

  it("saves the capture-aware sync setting when toggled", async () => {
    const user = userEvent.setup();
    render(SettingsPage);
    const checkbox = await screen.findByRole("checkbox", { name: /Capture-aware/i });
    await user.click(checkbox);
    expect(invoke).toHaveBeenCalledWith(
      "settings_set",
      expect.objectContaining({ key: "capture-aware", value: "true" }),
    );
  });

  it("saves the debug logging setting when toggled", async () => {
    const user = userEvent.setup();
    render(SettingsPage);
    const checkbox = await screen.findByRole("checkbox", { name: /Debug Logging/i });
    await user.click(checkbox);
    expect(invoke).toHaveBeenCalledWith(
      "settings_set",
      expect.objectContaining({ key: "debug-logging", value: "true" }),
    );
  });

  it("saves the home page selection", async () => {
    const user = userEvent.setup();
    render(SettingsPage);
    const select = (await screen.findByLabelText("Home page")) as HTMLSelectElement;
    await user.selectOptions(select, "favorites");
    expect(invoke).toHaveBeenCalledWith(
      "settings_set",
      expect.objectContaining({ key: "home-page", value: "favorites" }),
    );
    expect(await screen.findByText("home-page saved")).toBeInTheDocument();
  });

  it("saves the 7-Zip console executable path setting", async () => {
    const user = userEvent.setup();
    render(SettingsPage);
    const input = await screen.findByPlaceholderText("e.g. C:\\Utilities\\7z or C:\\Utilities\\7z\\7za.exe");
    await user.type(input, "C:\\Utilities\\7z\\7za.exe");
    await user.keyboard("{Enter}");

    expect(invoke).toHaveBeenCalledWith(
      "settings_set",
      expect.objectContaining({ key: "7z-path", value: "C:\\Utilities\\7z\\7za.exe" }),
    );
    expect(await screen.findByText("7z-path saved")).toBeInTheDocument();
  });

  it("shows Key saved status when sgdb-api-key is already set", async () => {
    render(SettingsPage);
    // The mock returns (set) for sgdb-api-key, so the badge should show.
    expect(await screen.findByText("● Key saved")).toBeInTheDocument();
  });

  it("saves the SteamGridDB API key as a secret", async () => {
    const user = userEvent.setup();
    render(SettingsPage);
    const input = await screen.findByPlaceholderText(
      "●●●●●●●●●●●●●●●● (saved — paste to replace)",
    );
    await user.type(input, "my-test-sgdb-key-1234");
    const saveBtn = await screen.findByRole("button", { name: "Save SteamGridDB API key" });
    await user.click(saveBtn);

    expect(invoke).toHaveBeenCalledWith(
      "settings_set",
      expect.objectContaining({
        key: "sgdb-api-key",
        value: "my-test-sgdb-key-1234",
        secret: true,
      }),
    );
  });
});
