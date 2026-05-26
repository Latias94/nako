import type {
  ContinueWatchingResponse,
  ItemDetailResponse,
  ItemsResponse,
  LibraryListResponse,
  LibrarySourcesResponse,
  MetadataProfileDto,
  SearchResponse,
} from "@nako/sdk";

const page = {
  limit: 20,
  offset: 0,
  returned: 2,
};

const metadataProfile: MetadataProfileDto = {
  country: "JP",
  image_providers: ["local"],
  item_kinds: ["series", "episode"],
  language: "ja",
  local_metadata_policy: "local_first",
  local_readers: ["nfo"],
  metadata_providers: ["local"],
  refresh_mode: "missing_only",
  scan: {
    addon_scrape: false,
    addon_writeback: false,
    enabled: true,
  },
};

export const fixtureLibraries: LibraryListResponse = {
  libraries: [
    {
      id: "library-anime",
      name: "Anime Vault",
      options: {
        domain: "video",
        metadata_profile: metadataProfile,
        naming_strategy: "anime",
        preset: "anime",
        scan: {
          max_depth: 8,
          realtime_monitor: true,
        },
      },
      roots: ["redacted-root"],
    },
    {
      id: "library-films",
      name: "Films",
      options: {
        domain: "video",
        metadata_profile: {
          ...metadataProfile,
          country: "US",
          item_kinds: ["movie"],
          language: "en",
        },
        naming_strategy: "movie",
        preset: "movies",
        scan: {
          max_depth: 4,
          realtime_monitor: false,
        },
      },
      roots: ["redacted-root"],
    },
  ],
  page,
};

export const fixtureItems: ItemsResponse = {
  items: [
    {
      id: "item-episode-1",
      kind: "episode",
      metadata: {
        collections: [],
        credits: [],
        external_ids: [],
        genres: ["Animation"],
        original_title: null,
        overview: "A carefully kept local episode.",
        ratings: [],
        release_date: "2026-01-10",
        runtime_minutes: 24,
        sort_title: "Pilot",
        studios: [],
        tagline: null,
        tags: ["fixture"],
        title: "Pilot",
      },
      parent_id: null,
    },
    {
      id: "item-film-1",
      kind: "movie",
      metadata: {
        collections: [],
        credits: [],
        external_ids: [],
        genres: ["Drama"],
        original_title: null,
        overview: "A local film record served by fixture data.",
        ratings: [],
        release_date: "2025-11-02",
        runtime_minutes: 118,
        sort_title: "After the Rain",
        studios: [],
        tagline: null,
        tags: ["fixture"],
        title: "After the Rain",
      },
      parent_id: null,
    },
  ],
  page,
};

export const fixtureContinueWatching: ContinueWatchingResponse = {
  items: [
    {
      images: [],
      item: fixtureItems.items[0],
      state: {
        duration_ms: 1_440_000,
        item_id: "item-episode-1",
        last_played_at: "2026-05-26T10:20:00Z",
        progress_percent: 0.38,
        resume_position_ms: 547_200,
        source_id: "source-episode-1",
        updated_at: "2026-05-26T10:20:00Z",
        version: 1,
        watched: false,
        watched_at: null,
      },
    },
  ],
  page: {
    limit: 20,
    offset: 0,
    returned: 1,
  },
};

export const fixtureLibrarySources: LibrarySourcesResponse = {
  library: fixtureLibraries.libraries[0],
  page: {
    limit: 20,
    offset: 0,
    returned: 1,
  },
  sources: [
    {
      item: fixtureItems.items[0],
      probe: {
        bit_rate: 8_200_000,
        container: "matroska",
        duration_ms: 1_440_000,
        streams: [
          {
            bit_rate: 7_800_000,
            channels: null,
            codec: "h264",
            duration_ms: 1_440_000,
            height: 1080,
            index: 0,
            kind: "video",
            language: null,
            sample_rate: null,
            width: 1920,
          },
          {
            bit_rate: 384_000,
            channels: 2,
            codec: "aac",
            duration_ms: 1_440_000,
            height: null,
            index: 1,
            kind: "audio",
            language: "ja",
            sample_rate: 48_000,
            width: null,
          },
        ],
      },
      source: {
        file_name: "Pilot.mkv",
        fingerprint: "redacted",
        id: "source-episode-1",
        item_id: "item-episode-1",
        library_id: "library-anime",
        size_bytes: 1_468_006_400,
      },
    },
  ],
};

export const fixtureSearch: SearchResponse = {
  hits: fixtureItems.items.map((item, index) => ({
    item,
    score: 1 - index * 0.1,
  })),
  page,
};

export const fixtureItemDetail: ItemDetailResponse = {
  collections: [],
  credits: [],
  genres: [],
  images: [],
  item: fixtureItems.items[0],
  sources: [
    {
      file_name: "Pilot.mkv",
      fingerprint: "redacted",
      id: "source-episode-1",
      item_id: "item-episode-1",
      library_id: "library-anime",
      size_bytes: 1_468_006_400,
    },
  ],
  studios: [],
  tags: [],
};
