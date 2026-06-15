import {
  NakoClient,
  type BrowserPlaybackTicketRequest,
  type BrowserPlaybackTicketResponse,
  type ContinueWatchingResponse,
  type FetchLike,
  type ItemDetailResponse,
  type ItemsResponse,
  type LibraryResponse,
  type LibraryListResponse,
  type LibraryItemsQuery,
  type LibrarySourcesResponse,
  type PageQuery,
  type PlaybackCapabilitiesQuery,
  type PlaybackDecisionResponse,
  type SearchResponse,
  type SetWatchedStateRequest,
  type UpdatePlaybackProgressRequest,
  type UserPlaybackStateDto,
  type UserPlaybackStateResponse,
} from "@nako/sdk";

import {
  fixtureContinueWatching,
  fixtureItemDetail,
  fixtureItems,
  fixtureLibraries,
  fixtureLibrarySources,
  fixturePlaybackDecision,
  fixtureSearch,
  fixtureUserPlaybackState,
} from "./fixtures";

const FIXTURE_PROGRESS_REPORTED_AT = "2026-05-26T10:30:00Z";
const FIXTURE_WATCHED_MARKED_AT = "2026-05-26T10:35:00Z";
const WATCHED_PROGRESS_PERCENT_THRESHOLD = 0.9;
const WATCHED_LONG_DURATION_MARGIN_MS = 120_000;

export type MediaConnection =
  | {
      mode: "live";
      baseUrl: string;
      bearerToken: string;
    }
  | {
      mode: "fixture";
    };

export type MediaSourceMode = "live" | "fixture";

export type MediaLoadResult<T> = {
  value: T;
  source: MediaSourceMode;
  error?: string;
};

export type MediaWebDataSource = {
  readonly source: MediaSourceMode;
  readonly label: string;
  checkConnection(): Promise<void>;
  listLibraries(page?: PageQuery): Promise<MediaLoadResult<LibraryListResponse>>;
  getLibrary(libraryId: string): Promise<MediaLoadResult<LibraryResponse>>;
  listLibrarySources(
    libraryId: string,
    page?: PageQuery,
  ): Promise<MediaLoadResult<LibrarySourcesResponse>>;
  listItems(query?: MediaItemsBrowseQuery): Promise<MediaLoadResult<ItemsResponse>>;
  searchItems(
    query: { facet?: string | string[]; q?: string } & PageQuery,
  ): Promise<MediaLoadResult<SearchResponse>>;
  getItem(itemId: string): Promise<MediaLoadResult<ItemDetailResponse>>;
  getPlaybackDecision(
    sourceId: string,
    capabilities?: PlaybackCapabilitiesQuery,
  ): Promise<MediaLoadResult<PlaybackDecisionResponse>>;
  createBrowserPlaybackTicket(
    sourceId: string,
    body: BrowserPlaybackTicketRequest,
  ): Promise<MediaLoadResult<BrowserPlaybackTicketResponse>>;
  getUserPlaybackState(itemId: string): Promise<MediaLoadResult<UserPlaybackStateResponse>>;
  updateUserPlaybackProgress(
    itemId: string,
    body: UpdatePlaybackProgressRequest,
  ): Promise<MediaLoadResult<UserPlaybackStateResponse>>;
  setUserWatchedState(
    itemId: string,
    body: SetWatchedStateRequest,
  ): Promise<MediaLoadResult<UserPlaybackStateResponse>>;
  listContinueWatching(page?: PageQuery): Promise<MediaLoadResult<ContinueWatchingResponse>>;
};

export type MediaDataSourceFactory = (connection: MediaConnection) => MediaWebDataSource;

export type MediaItemsBrowseQuery = PageQuery &
  Pick<LibraryItemsQuery, "facet" | "order" | "sort" | "watch_state">;

export function createMediaWebDataSource(connection: MediaConnection): MediaWebDataSource {
  if (connection.mode === "fixture") {
    return createFixtureMediaDataSource();
  }

  return createPublicClientMediaDataSource(connection);
}

export function createPublicClientMediaDataSource(
  connection: Extract<MediaConnection, { mode: "live" }>,
  fetch?: FetchLike,
): MediaWebDataSource {
  const client = new NakoClient({
    baseUrl: connection.baseUrl,
    bearerToken: connection.bearerToken,
    fetch,
  });

  return {
    source: "live",
    label: "Live Public Client API",
    async checkConnection() {
      await client.listLibraries({ limit: 1, offset: 0 });
    },
    async listLibraries(page = defaultPage()) {
      return liveResult(await client.listLibraries(page));
    },
    async getLibrary(libraryId) {
      return liveResult(await client.getLibrary(libraryId));
    },
    async listLibrarySources(libraryId, page = defaultPage()) {
      return liveResult(await client.listLibrarySources(libraryId, page));
    },
    async listItems(query = defaultPage()) {
      return liveResult(await client.listItems(toTopLevelItemsPageQuery(query)));
    },
    async searchItems(query) {
      return liveResult(await client.searchItems({ limit: 20, offset: 0, ...query }));
    },
    async getItem(itemId) {
      return liveResult(await client.getItem(itemId));
    },
    async getPlaybackDecision(sourceId, capabilities) {
      return liveResult(await client.getPlaybackDecision(sourceId, capabilities));
    },
    async createBrowserPlaybackTicket(sourceId, body) {
      return liveResult(
        resolveBrowserPlaybackUrls(
          await client.createBrowserPlaybackTicket(sourceId, body),
          connection.baseUrl,
        ),
      );
    },
    async getUserPlaybackState(itemId) {
      return liveResult(await client.getUserPlaybackState(itemId));
    },
    async updateUserPlaybackProgress(itemId, body) {
      return liveResult(await client.updateUserPlaybackProgress(itemId, body));
    },
    async setUserWatchedState(itemId, body) {
      return liveResult(await client.setUserWatchedState(itemId, body));
    },
    async listContinueWatching(page = defaultPage()) {
      return liveResult(await client.listContinueWatching(page));
    },
  };
}

export function createFixtureMediaDataSource(): MediaWebDataSource {
  const playbackStates = new Map<string, UserPlaybackStateDto>([
    [fixtureUserPlaybackState.state.item_id, { ...fixtureUserPlaybackState.state }],
  ]);

  const playbackStateFor = (itemId: string) =>
    playbackStates.get(itemId) ?? fixtureDefaultUserPlaybackState(itemId);

  const storePlaybackState = (state: UserPlaybackStateDto) => {
    playbackStates.set(state.item_id, state);
    return fixtureResult({ state });
  };

  return {
    source: "fixture",
    label: "Fixture mode",
    async checkConnection() {
      return;
    },
    async listLibraries() {
      return fixtureResult(fixtureLibraries);
    },
    async getLibrary(libraryId) {
      const library = fixtureLibraries.libraries.find((candidate) => candidate.id === libraryId);
      if (!library) {
        throw new Error("Media Library not found");
      }
      return fixtureResult({ library });
    },
    async listLibrarySources(libraryId) {
      if (fixtureLibrarySources.library.id !== libraryId) {
        throw new Error("Media Library sources not found");
      }
      return fixtureResult(fixtureLibrarySources);
    },
    async listItems() {
      return fixtureResult(fixtureItems);
    },
    async searchItems() {
      return fixtureResult(fixtureSearch);
    },
    async getItem() {
      return fixtureResult(fixtureItemDetail);
    },
    async getPlaybackDecision(sourceId) {
      return fixtureResult(fixturePlaybackDecision(sourceId));
    },
    async createBrowserPlaybackTicket(sourceId, body) {
      return fixtureResult(fixtureBrowserPlaybackTicket(sourceId, body));
    },
    async getUserPlaybackState(itemId) {
      return fixtureResult({ state: playbackStateFor(itemId) });
    },
    async updateUserPlaybackProgress(itemId, body) {
      const existing = playbackStateFor(itemId);
      const durationMs = body.duration_ms ?? existing.duration_ms;
      const reportedAt = body.reported_at ?? FIXTURE_PROGRESS_REPORTED_AT;
      const watched = existing.watched || fixtureIsWatchedByPolicy(body.position_ms, durationMs);
      const resumePositionMs = watched || body.position_ms === 0 ? null : body.position_ms;
      const state: UserPlaybackStateDto = {
        ...existing,
        duration_ms: durationMs,
        item_id: itemId,
        last_played_at: body.position_ms > 0 ? reportedAt : existing.last_played_at,
        progress_percent: fixturePlaybackProgressPercent(resumePositionMs, durationMs),
        resume_position_ms: resumePositionMs,
        source_id: body.source_id ?? existing.source_id,
        updated_at: reportedAt,
        version: existing.version + 1,
        watched,
        watched_at: watched
          ? (existing.watched_at ?? reportedAt)
          : null,
      };
      return storePlaybackState(state);
    },
    async setUserWatchedState(itemId, body) {
      const existing = playbackStateFor(itemId);
      const durationMs = body.duration_ms ?? existing.duration_ms;
      const positionMs = body.position_ms ?? existing.resume_position_ms;
      const markedAt = body.marked_at ?? FIXTURE_WATCHED_MARKED_AT;
      const resumePositionMs =
        body.watched || !positionMs || fixtureIsWatchedByPolicy(positionMs, durationMs)
          ? null
          : positionMs;
      const state: UserPlaybackStateDto = {
        ...existing,
        duration_ms: durationMs,
        item_id: itemId,
        last_played_at:
          positionMs && positionMs > 0 ? markedAt : existing.last_played_at,
        progress_percent: fixturePlaybackProgressPercent(resumePositionMs, durationMs),
        resume_position_ms: resumePositionMs,
        source_id: body.source_id ?? existing.source_id,
        updated_at: markedAt,
        version: existing.version + 1,
        watched: body.watched,
        watched_at: body.watched ? markedAt : null,
      };
      return storePlaybackState(state);
    },
    async listContinueWatching(page = defaultPage()) {
      return fixtureResult(fixtureContinueWatchingFromPlaybackStates(playbackStates, page));
    },
  };
}

function defaultPage(): PageQuery {
  return { limit: 20, offset: 0 };
}

function toTopLevelItemsPageQuery(query: PageQuery): PageQuery {
  return {
    limit: query.limit,
    offset: query.offset,
  };
}

function liveResult<T>(value: T): MediaLoadResult<T> {
  return {
    source: "live",
    value,
  };
}

function resolveBrowserPlaybackUrls(
  ticket: BrowserPlaybackTicketResponse,
  baseUrl: string,
): BrowserPlaybackTicketResponse {
  return {
    ...ticket,
    urls: ticket.urls.map((url) => ({
      ...url,
      url: new URL(url.url, baseUrl).toString(),
    })),
  };
}

function fixtureBrowserPlaybackTicket(
  sourceId: string,
  body: BrowserPlaybackTicketRequest,
): BrowserPlaybackTicketResponse {
  const encodedSourceId = encodeURIComponent(sourceId);
  const streamPath =
    body.mode === "hls"
      ? `/sources/${encodedSourceId}/stream/hls/playlist.m3u8`
      : body.mode === "remux"
        ? `/sources/${encodedSourceId}/stream/remux?output_container=${
            body.capabilities?.output_container ?? "mp4"
          }`
        : `/sources/${encodedSourceId}/stream`;
  const separator = streamPath.includes("?") ? "&" : "?";
  const isPlaylist = body.mode === "hls";

  return {
    expires_at: "2026-05-26T12:00:00Z",
    item_id: "item-episode-1",
    mode: body.mode,
    playback_session_id: body.mode === "hls" ? "playback-session-hls-fixture" : null,
    source_id: sourceId,
    urls: [
      {
        content_type: isPlaylist ? "application/vnd.apple.mpegurl" : "video/mp4",
        kind: isPlaylist ? "playlist" : "stream",
        supports_range_requests: !isPlaylist,
        url: `https://fixture.nako.test${streamPath}${separator}ticket=nako_bpt_fixture`,
      },
    ],
  };
}

function fixtureDefaultUserPlaybackState(itemId: string): UserPlaybackStateDto {
  return {
    duration_ms: null,
    item_id: itemId,
    last_played_at: null,
    progress_percent: null,
    resume_position_ms: null,
    source_id: null,
    updated_at: null,
    version: 0,
    watched: false,
    watched_at: null,
  };
}

function fixtureContinueWatchingFromPlaybackStates(
  playbackStates: ReadonlyMap<string, UserPlaybackStateDto>,
  page: PageQuery,
): ContinueWatchingResponse {
  const limit = page.limit ?? 20;
  const offset = page.offset ?? 0;
  const items = Array.from(playbackStates.values())
    .filter((state) => !state.watched && (state.resume_position_ms ?? 0) > 0)
    .sort((left, right) => compareIsoDescending(left.updated_at, right.updated_at))
    .flatMap((state) => {
      const fixtureEntry = fixtureContinueWatching.items.find(
        (entry) => entry.item.id === state.item_id,
      );
      if (!fixtureEntry) {
        return [];
      }

      return [
        {
          ...fixtureEntry,
          state: {
            ...state,
            progress_percent: fixturePlaybackProgressPercent(
              state.resume_position_ms,
              state.duration_ms,
            ),
          },
        },
      ];
    })
    .slice(offset, offset + limit);

  return {
    items,
    page: {
      limit,
      offset,
      returned: items.length,
    },
  };
}

function fixturePlaybackProgressPercent(
  positionMs: number | null | undefined,
  durationMs: number | null | undefined,
) {
  if (!positionMs || !durationMs || durationMs <= 0) {
    return null;
  }

  return Math.min(1, Math.max(0, positionMs / durationMs));
}

function fixtureIsWatchedByPolicy(
  positionMs: number,
  durationMs: number | null | undefined,
) {
  if (!durationMs || durationMs <= 0) {
    return false;
  }

  return (
    (durationMs >= 60_000 &&
      positionMs / durationMs >= WATCHED_PROGRESS_PERCENT_THRESHOLD) ||
    (durationMs >= 20 * 60_000 &&
      durationMs - positionMs <= WATCHED_LONG_DURATION_MARGIN_MS)
  );
}

function compareIsoDescending(left: string | null, right: string | null) {
  return (right ?? "").localeCompare(left ?? "");
}

function fixtureResult<T>(value: T): MediaLoadResult<T> {
  return {
    source: "fixture",
    value,
  };
}
