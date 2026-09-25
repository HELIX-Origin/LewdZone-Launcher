/**
 * Centralized in-memory client cache for LewdZone Launcher.
 *
 * Prevents redundant full-page reloads and state wipes when navigating between
 * Store, Library, Favorites, Queue, and Settings.
 * Pages render instantly from cache and only refresh when needed or commanded to.
 */

export interface GameCard {
  slug: string;
  post_id: number | null;
  title: string;
  thumb_url: string | null;
  platforms: string[];
  engine: string | null;
  state: string | null;
  version_tag: string | null;
  developer: string | null;
  genres: string[];
  genre_slugs: string[];
  external_genres: string[];
  views: number | null;
}

export interface Genre {
  label: string;
  slug: string;
  count: number | null;
}

export interface DownloadEntry {
  label: string;
  variant: string | null;
  host: string;
  platform: string | null;
  go_link: string;
}

export interface Version {
  label: string;
  is_latest: boolean;
  official: DownloadEntry[];
  community: DownloadEntry[];
}

export interface GameData {
  slug: string;
  post_id: number | null;
  title: string;
  developer: string | null;
  current_version: string | null;
  engine: string | null;
  platforms: string[];
  genres: string[];
  size_label: string | null;
  censorship: string | null;
  screenshots: string[];
  description: string | null;
  rating?: number | null;
  versions: Version[];
  download_entries: DownloadEntry[];
}

export interface LibraryGame {
  slug: string;
  post_id: number | null;
  title: string;
  version: string;
  platform: string;
  tab: string;
  engine: string | null;
  install_path: string;
  candidates: string[];
  launch_exe: string;
  installed_at: string | null;
  size_on_disk: number;
  playtime_seconds?: number;
  play_count?: number;
  last_played_at?: string | null;
  thumb_url?: string | null;
}

export interface CatalogFilterState {
  q: string;
  platform: string;
  engine: string;
  devState: string;
  sort: string;
  includeTags: string[];
  excludeTags: string[];
}

// ── 1. Store Catalog Cache ──────────────────────────────────────────────────
export const catalogStore = {
  games: [] as GameCard[],
  coverUrls: {} as Record<string, string | null>,
  totalPages: null as number | null,
  page: 1,
  genres: [] as Genre[],
  filters: {
    q: "",
    platform: "",
    engine: "",
    devState: "",
    sort: "Popularity",
    includeTags: [] as string[],
    excludeTags: [] as string[],
  } as CatalogFilterState,
  isLoaded: false,
};

// ── 2. Individual Game Details Cache ────────────────────────────────────────
export const gameDetailsCache = new Map<string, GameData>();

// ── 3. Library Cache ────────────────────────────────────────────────────────
export const libraryStore = {
  games: [] as LibraryGame[],
  coverUrls: {} as Record<string, string | null>,
  favorites: {} as Record<string, boolean>,
  root: null as string | null,
  isLoaded: false,
};

// ── 4. Favorites Cache ──────────────────────────────────────────────────────
export const favoritesStore = {
  games: [] as GameCard[],
  coverUrls: {} as Record<string, string | null>,
  isLoaded: false,
};

export const settingsStore = {
  snapshot: {} as Record<string, unknown>,
  themes: [] as string[],
  activeTheme: "(default)",
  logPath: null as string | null,
  isLoaded: false,
};

export function resetClientCache() {
  catalogStore.games = [];
  catalogStore.coverUrls = {};
  catalogStore.totalPages = null;
  catalogStore.page = 1;
  catalogStore.genres = [];
  catalogStore.filters = {
    q: "",
    platform: "",
    engine: "",
    devState: "",
    sort: "Popularity",
    includeTags: [],
    excludeTags: [],
  };
  catalogStore.isLoaded = false;

  gameDetailsCache.clear();

  libraryStore.games = [];
  libraryStore.coverUrls = {};
  libraryStore.favorites = {};
  libraryStore.root = null;
  libraryStore.isLoaded = false;

  favoritesStore.games = [];
  favoritesStore.coverUrls = {};
  favoritesStore.isLoaded = false;

  settingsStore.snapshot = {};
  settingsStore.themes = [];
  settingsStore.activeTheme = "(default)";
  settingsStore.logPath = null;
  settingsStore.isLoaded = false;
}
