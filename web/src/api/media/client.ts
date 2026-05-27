import {
  NakoClient,
  type BrowserPlaybackTicketRequest,
  type BrowserPlaybackTicketResponse,
  type ContinueWatchingResponse,
  type FetchLike as PublicClientFetch,
  type ItemDetailResponse,
  type LibraryListResponse,
  type LibraryResponse,
  type LibrarySourcesResponse,
  type ManagementContextLinksResponse,
  type PageQuery,
  type SearchResponse,
} from "@nako/sdk";

import {
  fixtureBrowserPlaybackTicket,
  fixtureContinueWatching,
  fixtureItemDetail,
  fixtureLibrarySources,
  fixtureManagementContextLinks,
  fixtureMediaLibrary,
  fixtureMediaLibraries,
  fixtureSearch,
} from "@/api/media/fixtures";
import {
  apiErrorMessage,
  fixtureResult,
  liveResult,
  normalizeBaseUrl,
  type ApiLoadResult,
} from "@/api/shared";

export type MediaApiConnection =
  | {
      mode: "fixture";
    }
  | {
      mode: "live";
      baseUrl: string;
      bearerToken?: string;
    };

export type MediaApi = {
  readonly source: "fixture" | "live";
  listLibraries(page?: PageQuery): Promise<ApiLoadResult<LibraryListResponse>>;
  getLibrary(libraryId: string): Promise<ApiLoadResult<LibraryResponse>>;
  listLibrarySources(
    libraryId: string,
    page?: PageQuery,
  ): Promise<ApiLoadResult<LibrarySourcesResponse>>;
  listContinueWatching(page?: PageQuery): Promise<ApiLoadResult<ContinueWatchingResponse>>;
  searchItems(
    query: { q?: string; facet?: string | string[] } & PageQuery,
  ): Promise<ApiLoadResult<SearchResponse>>;
  getItem(itemId: string): Promise<ApiLoadResult<ItemDetailResponse>>;
  managementContextLinks(query?: {
    item_id?: string;
    library_id?: string;
    playback_session_id?: string;
    source_id?: string;
  }): Promise<ApiLoadResult<ManagementContextLinksResponse>>;
  createBrowserPlaybackTicket(
    sourceId: string,
    body: BrowserPlaybackTicketRequest,
  ): Promise<ApiLoadResult<BrowserPlaybackTicketResponse>>;
};

export function createMediaApi(
  connection: MediaApiConnection = { mode: "fixture" },
  fetcher?: PublicClientFetch,
): MediaApi {
  if (connection.mode === "fixture") {
    return createFixtureMediaApi();
  }

  return createLiveMediaApi(connection, fetcher);
}

function createLiveMediaApi(
  connection: Extract<MediaApiConnection, { mode: "live" }>,
  fetcher?: PublicClientFetch,
): MediaApi {
  const baseUrl = normalizeBaseUrl(connection.baseUrl);
  const client = new NakoClient({
    baseUrl,
    bearerToken: connection.bearerToken,
    fetch: fetcher,
  });

  return {
    source: "live",
    async listLibraries(page) {
      return loadLive(() => client.listLibraries(page), fixtureMediaLibraries);
    },
    async getLibrary(libraryId) {
      return loadLive(() => client.getLibrary(libraryId), fixtureMediaLibrary);
    },
    async listLibrarySources(libraryId, page) {
      return loadLive(() => client.listLibrarySources(libraryId, page), fixtureLibrarySources);
    },
    async listContinueWatching(page) {
      return loadLive(() => client.listContinueWatching(page), fixtureContinueWatching);
    },
    async searchItems(query) {
      return loadLive(() => client.searchItems(query), fixtureSearch);
    },
    async getItem(itemId) {
      return loadLive(() => client.getItem(itemId), fixtureItemDetail);
    },
    async managementContextLinks(query) {
      return loadLive(() => client.managementContextLinks(query), fixtureManagementContextLinks);
    },
    async createBrowserPlaybackTicket(sourceId, body) {
      return loadLive(
        async () =>
          resolveBrowserPlaybackUrls(await client.createBrowserPlaybackTicket(sourceId, body), baseUrl),
        fixtureBrowserPlaybackTicket(sourceId, body),
      );
    },
  };
}

function createFixtureMediaApi(): MediaApi {
  return {
    source: "fixture",
    async listLibraries() {
      return fixtureResult(fixtureMediaLibraries);
    },
    async getLibrary(libraryId) {
      const library = fixtureMediaLibraries.libraries.find((candidate) => candidate.id === libraryId);
      if (!library) {
        throw new Error("Media Library not found");
      }

      return fixtureResult({ library });
    },
    async listLibrarySources(libraryId) {
      if (libraryId !== fixtureLibrarySources.library.id) {
        throw new Error("Media Library sources not found");
      }

      return fixtureResult(fixtureLibrarySources);
    },
    async listContinueWatching() {
      return fixtureResult(fixtureContinueWatching);
    },
    async searchItems() {
      return fixtureResult(fixtureSearch);
    },
    async getItem() {
      return fixtureResult(fixtureItemDetail);
    },
    async managementContextLinks() {
      return fixtureResult(fixtureManagementContextLinks);
    },
    async createBrowserPlaybackTicket(sourceId, body) {
      return fixtureResult(fixtureBrowserPlaybackTicket(sourceId, body));
    },
  };
}

async function loadLive<T>(loader: () => Promise<T>, fallback: T): Promise<ApiLoadResult<T>> {
  try {
    return liveResult(await loader());
  } catch (error: unknown) {
    return fixtureResult(fallback, apiErrorMessage(error, "Public Client API request failed"));
  }
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
