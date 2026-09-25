import { describe, it, expect } from "vitest";
import {
  toTitleCase,
  cleanDisplayTitle,
  formatReleaseDescriptor,
  formatCanonicalTitle,
  formatDisplayMessage,
} from "../format";

describe("format utilities", () => {
  describe("toTitleCase", () => {
    it("converts kebab-case to Title Case", () => {
      expect(toTitleCase("dating-my-daughter")).toBe("Dating My Daughter");
      expect(toTitleCase("wild-life")).toBe("Wild Life");
      expect(toTitleCase("treasure_of_nadia")).toBe("Treasure Of Nadia");
    });
  });

  describe("cleanDisplayTitle", () => {
    it("converts raw slugs into title case", () => {
      expect(cleanDisplayTitle("dating-my-daughter")).toBe("Dating My Daughter");
    });

    it("cleans brackets and release markers", () => {
      expect(cleanDisplayTitle("Dating My Daughter [Unknown] - Version Ch. 1-4 v1.01")).toBe(
        "Dating My Daughter"
      );
      expect(cleanDisplayTitle("Treasure of Nadia [Finished] - Version 1.0117")).toBe(
        "Treasure of Nadia"
      );
      expect(cleanDisplayTitle("Wild Life [Ongoing] - Version 2026-06-15 - PC")).toBe(
        "Wild Life"
      );
      expect(cleanDisplayTitle("Elysium [Ongoing] - Chapter 3")).toBe("Elysium");
    });
  });

  describe("formatReleaseDescriptor", () => {
    it("correctly separates version and chapter", () => {
      expect(formatReleaseDescriptor("Dating My Daughter [Unknown]", "Ch. 1-4 v1.01")).toBe(
        "Version 1.01 Chapter 1-4"
      );
    });

    it("handles version only", () => {
      expect(formatReleaseDescriptor("Harem Hotel", "v0.19.1")).toBe("Version 0.19.1");
      expect(formatReleaseDescriptor("Lost City", "1.02")).toBe("Version 1.02");
    });

    it("handles chapter only", () => {
      expect(formatReleaseDescriptor("Elysium", "Chapter 3")).toBe("Chapter 3");
      expect(formatReleaseDescriptor("Anime Game", "Ch. 4")).toBe("Chapter 4");
    });
  });

  describe("formatCanonicalTitle", () => {
    it("produces correct canonical title for games with version and chapter", () => {
      expect(
        formatCanonicalTitle("Dating My Daughter [Unknown] - Version Ch. 1-4 v1.01")
      ).toBe("Dating My Daughter [Unknown] - Version 1.01 Chapter 1-4");

      expect(
        formatCanonicalTitle("Dating My Daughter", "Unknown", "Ch. 1-4 v1.01")
      ).toBe("Dating My Daughter [Unknown] - Version 1.01 Chapter 1-4");
    });

    it("handles finished and ongoing statuses", () => {
      expect(
        formatCanonicalTitle("Treasure of Nadia", "Finished", "v1.0117")
      ).toBe("Treasure of Nadia [Finished] - Version 1.0117");

      expect(
        formatCanonicalTitle("Wild Life", "Ongoing", "v2026-06-15")
      ).toBe("Wild Life [Ongoing] - Version 2026-06-15");
    });
  });

  describe("formatDisplayMessage", () => {
    it("formats archive/title message strings", () => {
      expect(
        formatDisplayMessage("Dating My Daughter [Unknown] - Version Ch. 1-4 v1.01")
      ).toBe("Dating My Daughter [Unknown] - Version 1.01 Chapter 1-4");
    });

    it("preserves non-title status messages", () => {
      expect(formatDisplayMessage("Streaming from intercept")).toBe("Streaming from intercept");
      expect(formatDisplayMessage("Fetching download link...")).toBe("Fetching download link...");
    });
  });
});
