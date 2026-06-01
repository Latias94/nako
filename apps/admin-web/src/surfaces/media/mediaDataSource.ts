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
  type LibrarySourcesResponse,
  type PageQuery,
  type PlaybackCapabilitiesQuery,
  type PlaybackDecisionResponse,
  type SearchResponse,
  type SetWatchedStateRequest,
  type UpdatePlaybackProgressRequest,
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
  listItems(page?: PageQuery): Promise<MediaLoadResult<ItemsResponse>>;
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
    async listItems(page = defaultPage()) {
      return liveResult(await client.listItems(page));
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
    async getUserPlaybackState() {
      return fixtureResult(fixtureUserPlaybackState);
    },
    async updateUserPlaybackProgress(itemId, body) {
      return fixtureResult({
        state: {
          ...fixtureUserPlaybackState.state,
          duration_ms: body.duration_ms ?? fixtureUserPlaybackState.state.duration_ms,
          item_id: itemId,
          last_played_at: body.reported_at ?? "2026-05-26T10:30:00Z",
          progress_percent: body.duration_ms ? body.position_ms / body.duration_ms : null,
          resume_position_ms: body.position_ms,
          source_id: body.source_id ?? fixtureUserPlaybackState.state.source_id,
          updated_at: body.reported_at ?? "2026-05-26T10:30:00Z",
          watched: false,
          watched_at: null,
          version: fixtureUserPlaybackState.state.version + 1,
        },
      });
    },
    async setUserWatchedState(itemId, body) {
      return fixtureResult({
        state: {
          ...fixtureUserPlaybackState.state,
          duration_ms: body.duration_ms ?? fixtureUserPlaybackState.state.duration_ms,
          item_id: itemId,
          progress_percent: body.watched ? 1 : fixtureUserPlaybackState.state.progress_percent,
          resume_position_ms:
            body.position_ms ?? fixtureUserPlaybackState.state.resume_position_ms,
          source_id: body.source_id ?? fixtureUserPlaybackState.state.source_id,
          updated_at: body.marked_at ?? "2026-05-26T10:35:00Z",
          version: fixtureUserPlaybackState.state.version + 1,
          watched: body.watched,
          watched_at: body.watched ? (body.marked_at ?? "2026-05-26T10:35:00Z") : null,
        },
      });
    },
    async listContinueWatching() {
      return fixtureResult(fixtureContinueWatching);
    },
  };
}

function defaultPage(): PageQuery {
  return { limit: 20, offset: 0 };
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

function fixtureResult<T>(value: T): MediaLoadResult<T> {
  return {
    source: "fixture",
    value,
  };
}
