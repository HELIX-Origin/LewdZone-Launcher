import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async () => "Hello, test! from Rust"),
}));;

vi.mock("$app/state", () => ({
  page: { url: new URL("http://localhost/store") },
}));

vi.mock("$app/navigation", () => ({
  goto: vi.fn(),
}));

import Layout from "../../routes/+layout.svelte";
import SettingsPage from "../../routes/settings/+page.svelte";
import StorePage from "../../routes/store/+page.svelte";

describe("app shell", () => {
  it("renders the four primary tabs", () => {
    render(Layout);
    for (const label of ["Store", "Library", "Downloads", "Settings"]) {
      expect(screen.getByRole("button", { name: label })).toBeInTheDocument();
    }
  });
});

describe("store page", () => {
  beforeEach(() => {
    const invokeMock = invoke as unknown as ReturnType<typeof vi.fn>;
    invokeMock.mockReset();
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === "catalog_page") {
        return {
          games: [
            {
              slug: "wild-life",
              title: "Wild Life",
              thumb_url: "https://h1.lzcdn.com/img/wild-life-cover.jpg",
              platforms: ["pc"],
              engine: "Unreal Engine",
              state: "Ongoing",
              version_tag: "v2026-06-15 Full",
              developer: "Adeptus Steve",
              genres: ["3D Game"],
              views: "962K",
            },
          ],
          meta: { page: 1, total_pages: 787 },
        };
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
});

describe("settings page", () => {

  beforeEach(() => {
    const invokeMock = invoke as unknown as ReturnType<typeof vi.fn>;
    invokeMock.mockReset();
    invokeMock.mockImplementation(
      async (cmd: string, args?: { key?: string; value?: string }) => {
        switch (cmd) {
          case "settings_get":
            return {
              "library-root": "D:/Games",
              "capture-aware": false,
              theme: "",
            };
          case "themes_list":
            return ["midnight", "candy"];
          case "settings_set":
            return { key: args?.key, value: args?.value };
          default:
            return [];
        }
      },
    );
  });

  it("loads snapshot + theme list into the page", async () => {
    render(SettingsPage);
    expect(await screen.findByText("Settings")).toBeInTheDocument();
    expect(await screen.findByDisplayValue("D:/Games")).toBeInTheDocument();
    expect(await screen.findByRole("option", { name: "midnight" })).toBeInTheDocument();
  });

  it("saves a text setting through settings_set", async () => {
    const user = userEvent.setup();
    render(SettingsPage);
    const list = await screen.findByDisplayValue("D:/Games");
    await user.type(list, "/SteamLibrary");
    await user.keyboard("{Enter}");

    expect(invoke).toHaveBeenCalledWith(
      "settings_set",
      expect.objectContaining({ key: "library-root", value: "D:/Games/SteamLibrary" }),
    );
    expect(await screen.findByText("library-root saved")).toBeInTheDocument();
  });
});