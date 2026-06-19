import type { ItemsResponse, SearchResponse } from "@nako/sdk";

import type {
  MediaItemsBrowseOrder,
  MediaItemsBrowseSearch,
  MediaItemsBrowseSort,
  MediaItemsWatchState,
  MediaSearchRouteSearch,
} from "./MediaCore";
import type {
  MediaItemsBrowseQuery,
  MediaLoadResult,
  MediaWebDataSource,
} from "./mediaDataSource";

const DEFAULT_LIMIT = 20;
const DEFAULT_OFFSET = 0;

export function mediaItemsBrowseActiveFilterCount(search: MediaItemsBrowseSearch) {
  return [search.facet, search.order, search.q, search.sort, search.watch_state].filter(
    (value) => value !== undefined,
  ).length;
}

export function mediaItemsBrowseHasPaginationDelta(search: MediaItemsBrowseSearch) {
  return search.limit !== DEFAULT_LIMIT || search.offset !== DEFAULT_OFFSET;
}

export function mediaItemsBrowseResetSearch(): Partial<MediaItemsBrowseSearch> {
  return {
    facet: undefined,
    limit: DEFAULT_LIMIT,
    offset: DEFAULT_OFFSET,
    order: undefined,
    q: undefined,
    sort: undefined,
    watch_state: undefined,
  };
}

export function mediaItemsBrowseSearchChange(value: string): Partial<MediaItemsBrowseSearch> {
  return { offset: DEFAULT_OFFSET, q: value || undefined };
}

export function mediaItemsBrowseFacetChange(value: string): Partial<MediaItemsBrowseSearch> {
  return { facet: value || undefined, offset: DEFAULT_OFFSET };
}

export function mediaItemsBrowseSortChange(value: string): Partial<MediaItemsBrowseSearch> {
  return { offset: DEFAULT_OFFSET, sort: mediaItemsBrowseSortValue(value) };
}

export function mediaItemsBrowseOrderChange(value: string): Partial<MediaItemsBrowseSearch> {
  return { offset: DEFAULT_OFFSET, order: mediaItemsBrowseOrderValue(value) };
}

export function mediaItemsBrowseWatchStateChange(
  value: string,
): Partial<MediaItemsBrowseSearch> {
  return { offset: DEFAULT_OFFSET, watch_state: mediaItemsBrowseWatchStateValue(value) };
}

export function mediaItemsBrowseLimitChange(
  value: string,
  fallback: number,
): Partial<MediaItemsBrowseSearch> {
  return { limit: positiveNumberInput(value, fallback, 1), offset: DEFAULT_OFFSET };
}

export function mediaSearchSubmitChange(query: string): Partial<MediaSearchRouteSearch> {
  return { offset: DEFAULT_OFFSET, q: query.trim() || undefined };
}

export function mediaItemsBrowseSortValue(value: string): MediaItemsBrowseSort | undefined {
  return value === "title" ||
    value === "release_date" ||
    value === "date_added" ||
    value === "last_played"
    ? value
    : undefined;
}

export function mediaItemsBrowseOrderValue(value: string): MediaItemsBrowseOrder | undefined {
  return value === "asc" || value === "desc" ? value : undefined;
}

export function mediaItemsBrowseWatchStateValue(
  value: string,
): MediaItemsWatchState | undefined {
  return value === "watched" || value === "unwatched" || value === "in_progress"
    ? value
    : undefined;
}

export async function loadMediaItems(
  source: MediaWebDataSource,
  search: MediaItemsBrowseSearch,
  errorMessage: string,
): Promise<MediaLoadResult<ItemsResponse>> {
  try {
    const result = search.q
      ? searchResultToItemsResult(await source.searchItems(buildMediaSearchQuery(search)))
      : await source.listItems(buildMediaItemsBrowseQuery(search));
    return {
      ...result,
      error: result.error ? errorMessage : undefined,
    };
  } catch {
    throw new Error(errorMessage);
  }
}

export async function loadLibraryItems(
  source: MediaWebDataSource,
  libraryId: string,
  search: MediaItemsBrowseSearch,
  errorMessage: string,
): Promise<MediaLoadResult<ItemsResponse>> {
  try {
    const result = await source.listLibraryItems(libraryId, buildMediaItemsBrowseQuery(search));
    return {
      ...result,
      error: result.error ? errorMessage : undefined,
    };
  } catch {
    throw new Error(errorMessage);
  }
}

export async function loadMediaSearch(
  source: MediaWebDataSource,
  search: MediaSearchRouteSearch,
  errorMessage: string,
): Promise<MediaLoadResult<SearchResponse>> {
  try {
    const result = await source.searchItems(buildMediaSearchQuery(search));
    return {
      ...result,
      error: result.error ? errorMessage : undefined,
    };
  } catch {
    throw new Error(errorMessage);
  }
}

function buildMediaItemsBrowseQuery(search: MediaItemsBrowseSearch): MediaItemsBrowseQuery {
  return {
    facet: search.facet,
    limit: search.limit,
    offset: search.offset,
    order: search.order,
    sort: search.sort,
    watch_state: search.watch_state,
  };
}

function buildMediaSearchQuery(
  search: MediaItemsBrowseSearch | MediaSearchRouteSearch,
): {
  facet?: string;
  limit: number;
  offset: number;
  q?: string;
} {
  return {
    facet: search.facet,
    limit: search.limit,
    offset: search.offset,
    q: search.q,
  };
}

function positiveNumberInput(value: string, fallback: number, minimum: number) {
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed >= minimum ? parsed : fallback;
}

function searchResultToItemsResult(
  result: MediaLoadResult<SearchResponse>,
): MediaLoadResult<ItemsResponse> {
  return {
    ...result,
    value: {
      items: result.value.hits.map((hit) => hit.item),
      page: result.value.page,
    },
  };
}
