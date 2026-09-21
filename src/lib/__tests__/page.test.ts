import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async () => "Hello, test! from Rust"),
}));

import Page from "../../routes/+page.svelte";

describe("+page.svelte", () => {
  it("renders the welcome heading", () => {
    render(Page);
    expect(
      screen.getByRole("heading", { name: /welcome to tauri \+ svelte/i }),
    ).toBeInTheDocument();
  });

  it("greets the user through the Rust core", async () => {
    const user = userEvent.setup();
    render(Page);
    const input = screen.getByPlaceholderText(/enter a name/i);
    const button = screen.getByRole("button", { name: /greet/i });

    await user.type(input, "test");
    await user.click(button);

    expect(invoke).toHaveBeenCalledWith("greet", { name: "test" });
    expect(await screen.findByText("Hello, test! from Rust")).toBeInTheDocument();
  });
});