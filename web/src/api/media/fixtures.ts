import type {
  BrowserPlaybackTicketRequest,
  BrowserPlaybackTicketResponse,
  ContinueWatchingResponse,
  ItemDetailResponse,
  LibraryListResponse,
  ManagementContextLinksResponse,
  SearchResponse,
} from "@nako/sdk";

const page = {
  limit: 20,
  offset: 0,
  returned: 2,
};

export const fixtureMediaLibraries: LibraryListResponse = {
  libraries: [
    {
      id: "library-films",
      name: "Films",
      options: {
        domain: "video",
        metadata_profile: {
          country: "US",
          image_providers: ["local"],
          item_kinds: ["movie"],
          language: "en",
          local_metadata_policy: "local_first",
          local_readers: ["nfo"],
          metadata_providers: ["local"],
          refresh_mode: "missing_only",
          scan: {
            addon_scrape: false,
            addon_writeback: false,
            enabled: true,
          },
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
    {
      id: "library-anime",
      name: "Anime",
      options: {
        domain: "video",
        metadata_profile: {
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
        },
        naming_strategy: "anime",
        preset: "anime",
        scan: {
          max_depth: 8,
          realtime_monitor: true,
        },
      },
      roots: ["redacted-root"],
    },
  ],
  page,
};

const fixtureItem = {
  id: "item-local-feature",
  kind: "movie" as const,
  metadata: {
    collections: [],
    credits: [],
    external_ids: [],
    genres: ["Drama"],
    original_title: null,
    overview: "A local media record served by the Nako fixture boundary.",
    ratings: [],
    release_date: "2026-01-10",
    runtime_minutes: 118,
    sort_title: "Local Feature",
    studios: [],
    tagline: null,
    tags: ["fixture"],
    title: "Local Feature",
  },
  parent_id: null,
};

export const fixtureContinueWatching: ContinueWatchingResponse = {
  items: [
    {
      images: [],
      item: fixtureItem,
      state: {
        duration_ms: 7_080_000,
        item_id: fixtureItem.id,
        last_played_at: "2026-05-28T01:00:00Z",
        progress_percent: 0.42,
        resume_position_ms: 2_973_600,
        source_id: "source-local-feature",
        updated_at: "2026-05-28T01:00:00Z",
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

export const fixtureSearch: SearchResponse = {
  hits: [
    {
      item: fixtureItem,
      score: 1,
    },
  ],
  page: {
    limit: 20,
    offset: 0,
    returned: 1,
  },
};

export const fixtureItemDetail: ItemDetailResponse = {
  collections: [],
  credits: [],
  genres: [],
  images: [],
  item: fixtureItem,
  sources: [
    {
      file_name: "Local Feature.mp4",
      fingerprint: "redacted",
      id: "source-local-feature",
      item_id: fixtureItem.id,
      library_id: "library-films",
      size_bytes: 1_126_400_000,
    },
  ],
  studios: [],
  tags: [],
};

export const fixtureManagementContextLinks: ManagementContextLinksResponse = {
  context: {
    item_id: fixtureItem.id,
    library_id: "library-films",
    playback_session_id: null,
    source_id: "source-local-feature",
  },
  links: [],
};

export function fixtureBrowserPlaybackTicket(
  sourceId: string,
  body: BrowserPlaybackTicketRequest,
): BrowserPlaybackTicketResponse {
  const encodedSourceId = encodeURIComponent(sourceId);
  const streamPath =
    body.mode === "hls"
      ? `/sources/${encodedSourceId}/stream/hls/playlist.m3u8`
      : body.mode === "remux"
        ? `/sources/${encodedSourceId}/stream/remux`
        : `/sources/${encodedSourceId}/stream`;

  return {
    expires_at: "2026-05-28T02:00:00Z",
    item_id: fixtureItem.id,
    mode: body.mode,
    source_id: sourceId,
    urls: [
      {
        content_type: body.mode === "hls" ? "application/vnd.apple.mpegurl" : "video/mp4",
        kind: body.mode === "hls" ? "playlist" : "stream",
        supports_range_requests: body.mode !== "hls",
        url: `https://fixture.nako.test${streamPath}?ticket=nako_bpt_fixture`,
      },
    ],
  };
}
