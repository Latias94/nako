import {
  NakoClient,
  type ContinueWatchingResponse,
  type FetchLike,
  type ItemDetailResponse,
  type ItemsResponse,
  type LibraryListResponse,
  type PageQuery,
  type SearchResponse,
} from "@nako/sdk";

import {
  fixtureContinueWatching,
  fixtureItemDetail,
  fixtureItems,
  fixtureLibraries,
  fixtureSearch,
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
  listItems(page?: PageQuery): Promise<MediaLoadResult<ItemsResponse>>;
  searchItems(query: { q?: string } & PageQuery): Promise<MediaLoadResult<SearchResponse>>;
  getItem(itemId: string): Promise<MediaLoadResult<ItemDetailResponse>>;
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
    async listItems(page = defaultPage()) {
      return liveResult(await client.listItems(page));
    },
    async searchItems(query) {
      return liveResult(await client.searchItems({ limit: 20, offset: 0, ...query }));
    },
    async getItem(itemId) {
      return liveResult(await client.getItem(itemId));
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
    async listItems() {
      return fixtureResult(fixtureItems);
    },
    async searchItems() {
      return fixtureResult(fixtureSearch);
    },
    async getItem() {
      return fixtureResult(fixtureItemDetail);
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
function fixtureResult<T>(value: T): MediaLoadResult<T> {
  return {
    source: "fixture",
    value,
  };
}
