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
    for (const label of ["Favorites", "Library", "Store", "Downloads", "Settings"]) {
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

  it("renders a custom menu bar with items and a profile button", () => {
    render(Layout);
    for (const label of ["File", "View", "Help"]) {
      expect(screen.getByRole("button", { name: label })).toBeInTheDocument();
    }
    expect(
      screen.getByRole("button", { name: "Profile" }),
    ).toBeInTheDocument();
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
    expect(screen.getByRole("button", { name: "Download Wild Life" })).toBeInTheDocument();
    expect(screen.getByLabelText("Source")).toBeInTheDocument();
  });

  it("defaults the source dropdown to the preferred source", async () => {
    mockInvoke(async (cmd: string) => {
      if (cmd === "game_page") return sampleGameData;
      if (cmd === "game_sources") return [...sampleSources, { host: "mega", label: "MEGA", preferred: true }];
      if (cmd === "content_enrich") {
        return { description: null, developer: null, rating: null, tags: [], screenshots: [], genres: [] };
      }
      return [];
    });
    render(GamePage);
    await screen.findByText("Wild Life");
    const select = await screen.findByLabelText("Source");
    expect((select as HTMLSelectElement).value).toBe("mega");
  });

  it("queues a download through game_download", async () => {
    const user = userEvent.setup();
    render(GamePage);
    const btn = await screen.findByRole("button", { name: "Download Wild Life" });
    await user.click(btn);
    expect(invoke).toHaveBeenCalledWith(
      "game_download",
      expect.objectContaining({
        slug: "wild-life",
        version: "v2026-06-15 Full",
        platform: "PC",
        tab: "official",
        source: "fileknot",
      }),
    );
    expect(
      await screen.findByText("Queued download #1. Watch the Downloads page for progress."),
    ).toBeInTheDocument();
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
      return [];
    });
    render(LibraryPage);
    const btn = await screen.findByRole("button", { name: /Add Wild Life to favorites/ });
    await user.click(btn);
    expect(invocations.some((i) => i.cmd === "favorite_add" && i.args?.slug === "wild-life")).toBe(true);
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

  it("shows the preferred download sources setting with the saved value", async () => {
    render(SettingsPage);
    const input = (await screen.findByDisplayValue("mega,google")) as HTMLInputElement;
    expect(input).toBeInTheDocument();
    expect(input.value).toBe("mega,google");
  });

  it("saves the preferred download sources setting", async () => {
    const user = userEvent.setup();
    render(SettingsPage);
    const input = (await screen.findByDisplayValue("mega,google")) as HTMLInputElement;
    await user.clear(input);
    await user.type(input, "dropbox,mega");
    await user.keyboard("{Enter}");

    expect(invoke).toHaveBeenCalledWith(
      "settings_set",
      expect.objectContaining({ key: "source-priority", value: "dropbox,mega" }),
    );
    expect(await screen.findByText("source-priority saved")).toBeInTheDocument();
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

  it("shows secret API keys as set or not set without revealing values", async () => {
    render(SettingsPage);
    expect(await screen.findByLabelText("SteamGridDB API key")).toBeInTheDocument();
    const sgdb = (await screen.findByLabelText("SteamGridDB API key")) as HTMLInputElement;
    expect(sgdb.placeholder).toBe("(set)");
    const igdbId = (await screen.findByLabelText("IGDB client ID")) as HTMLInputElement;
    expect(igdbId.placeholder).toBe("(not set)");
    expect(sgdb.value).toBe("");
  });

  it("saves an API key via settings_set with secret=true", async () => {
    const user = userEvent.setup();
    render(SettingsPage);
    const input = (await screen.findByLabelText("IGDB client secret")) as HTMLInputElement;
    await user.type(input, "my-secret-value");
    await user.keyboard("{Enter}");

    expect(invoke).toHaveBeenCalledWith(
      "settings_set",
      expect.objectContaining({ key: "igdb-client-secret", value: "my-secret-value", secret: true }),
    );
    expect(await screen.findByText("igdb-client-secret saved")).toBeInTheDocument();
  });
});