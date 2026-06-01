import type {
  ContinueWatchingResponse,
  ItemDetailResponse,
  ItemsResponse,
  LibraryListResponse,
  LibrarySourcesResponse,
  MediaStreamDispositionDto,
  MetadataProfileDto,
  PlaybackDecisionResponse,
  SearchResponse,
  UserPlaybackStateResponse,
} from "@nako/sdk";

const page = {
  limit: 20,
  offset: 0,
  returned: 2,
};

const streamDisposition: MediaStreamDispositionDto = {
  attached_pic: false,
  captions: false,
  commentary: false,
  default: false,
  descriptions: false,
  forced: false,
  hearing_impaired: false,
  visual_impaired: false,
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
            origin: null,
            disposition: streamDisposition,
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
            origin: null,
            disposition: streamDisposition,
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
    {
      file_name: "Pilot.alt.mp4",
      fingerprint: "redacted",
      id: "source-episode-1-alt",
      item_id: "item-episode-1",
      library_id: "library-anime",
      size_bytes: 1_126_400_000,
    },
  ],
  studios: [],
  tags: [],
};

export const fixtureUserPlaybackState: UserPlaybackStateResponse = {
  state: fixtureContinueWatching.items[0].state,
};

export function fixturePlaybackDecision(sourceId: string): PlaybackDecisionResponse {
  const source =
    fixtureItemDetail.sources.find((candidate) => candidate.id === sourceId) ??
    fixtureItemDetail.sources[0];
  const isMp4 = source.file_name.endsWith(".mp4");

  return {
    decision: {
      denial: null,
      direct_play: {
        content_type: isMp4 ? "video/mp4" : "video/x-matroska",
        source_id: source.id,
        supports_range_requests: true,
      },
      mode: "direct_play",
      reason: "compatible",
      report: {
        denial: null,
        direct_play: {
          reasons: ["compatible"],
          supported: true,
        },
        remux: {
          reasons: ["container_unsupported"],
          supported: false,
        },
        selected_mode: "direct_play",
        transcode: {
          reasons: ["requested_transcode_output"],
          supported: false,
        },
      },
      transcode_plan: null,
    },
    probe: {
      bit_rate: isMp4 ? 6_100_000 : 8_200_000,
      container: isMp4 ? "mp4" : "matroska",
      duration_ms: 1_440_000,
      streams: [
        {
          bit_rate: isMp4 ? 5_700_000 : 7_800_000,
          channels: null,
          codec: "h264",
          duration_ms: 1_440_000,
          height: isMp4 ? 720 : 1080,
          index: 0,
          kind: "video",
          language: null,
          origin: null,
          disposition: streamDisposition,
          sample_rate: null,
          width: isMp4 ? 1280 : 1920,
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
          origin: null,
          disposition: streamDisposition,
          sample_rate: 48_000,
          width: null,
        },
      ],
    },
    source,
    target: {
      control_capabilities: {
        commands: [],
      },
      kind: "browser",
      media_capabilities: {
        audio_codecs: ["aac", "mp3", "opus"],
        containers: ["mp4", "m4v", "webm"],
        direct_play: true,
        video_codecs: ["h264", "hevc", "vp9"],
      },
      network_scope: "local",
      transport_auth: "browser_ticket",
    },
  };
}
