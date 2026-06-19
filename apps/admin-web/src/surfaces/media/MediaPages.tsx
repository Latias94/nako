import { Link } from "@tanstack/react-router";
import type {
  ContinueWatchingResponse,
  ItemsResponse,
  LibraryListResponse,
  LibraryResponse,
  LibrarySourcesResponse,
  MediaItemDto,
  SearchResponse,
} from "@nako/sdk";
import { ArrowLeft, ArrowRight, Play, RotateCcw, Search, X } from "lucide-react";
import { useEffect, useState } from "react";

import { Button } from "../../components/ui/Button";
import { FilterActions, FilterBar, FilterField } from "../../components/ui/FilterBar";
import {
  formatBytes,
  formatRuntimeMinutes,
  formatRuntimeMs,
  useMediaLoad,
  type MediaAsyncState,
  type MediaItemsBrowseSearch,
  type MediaPageSearch,
  type MediaSearchChange,
  type MediaSearchRouteSearch,
} from "./MediaCore";
import {
  loadLibraryItems,
  loadMediaItems,
  loadMediaSearch,
  mediaItemsBrowseActiveFilterCount,
  mediaItemsBrowseFacetChange,
  mediaItemsBrowseHasPaginationDelta,
  mediaItemsBrowseLimitChange,
  mediaItemsBrowseOrderChange,
  mediaItemsBrowseResetSearch,
  mediaItemsBrowseSearchChange,
  mediaItemsBrowseSortChange,
  mediaItemsBrowseWatchStateChange,
  mediaSearchSubmitChange,
} from "./mediaBrowsePlanner";
import { MediaConnectPage } from "./MediaConnectPage";
import { useMediaSession } from "./MediaSession";
import type { MediaWebDataSource } from "./mediaDataSource";

export type {
  MediaItemSearch,
  MediaItemsBrowseSearch,
  MediaPageSearch,
  MediaSearchRouteSearch,
} from "./MediaCore";

export { MediaConnectPage } from "./MediaConnectPage";

const RECENTLY_ADDED_ITEMS_ERROR = "Recently added media could not be loaded.";
const MEDIA_ITEMS_ERROR = "Media items could not be loaded.";
const LIBRARY_ITEMS_ERROR = "Library items could not be loaded.";
const MEDIA_SEARCH_ERROR = "Search results could not be loaded.";

export function MediaHomePage() {
  const { dataSource } = useMediaSession();
  const [continueWatchingRefreshKey, setContinueWatchingRefreshKey] = useState(0);
  const [clearingContinueWatchingItemId, setClearingContinueWatchingItemId] =
    useState<string | null>(null);
  const [continueWatchingMutationError, setContinueWatchingMutationError] =
    useState<string | null>(null);
  const continueWatching = useMediaLoad(
    dataSource,
    (source) => source.listContinueWatching(),
    [continueWatchingRefreshKey],
  );
  const items = useMediaLoad(dataSource, (source) =>
    loadMediaItems(source, { limit: 8, offset: 0 }, RECENTLY_ADDED_ITEMS_ERROR),
  );

  async function startContinueWatchingOver(entry: ContinueWatchingResponse["items"][number]) {
    if (!dataSource) {
      return;
    }

    setClearingContinueWatchingItemId(entry.item.id);
    setContinueWatchingMutationError(null);
    try {
      await dataSource.setUserWatchedState(entry.item.id, {
        duration_ms: entry.state.duration_ms,
        position_ms: 0,
        ...(entry.state.source_id ? { source_id: entry.state.source_id } : {}),
        watched: false,
      });
      setContinueWatchingRefreshKey((current) => current + 1);
    } catch {
      setContinueWatchingMutationError("Continue Watching progress could not be cleared.");
    } finally {
      setClearingContinueWatchingItemId(null);
    }
  }

  if (!dataSource) {
    return <MediaConnectPage />;
  }

  return (
    <section className="mediaPage" aria-labelledby="media-home-title">
      <header className="mediaPageHeader">
        <h2 id="media-home-title">Watch next</h2>
      </header>
      <MediaContinueWatching
        clearingItemId={clearingContinueWatchingItemId}
        error={continueWatchingMutationError}
        onStartOver={(entry) => void startContinueWatchingOver(entry)}
        result={continueWatching}
      />
      <section className="mediaPanel" aria-labelledby="media-items-title">
        <div className="mediaPanelHeader">
          <h3 id="media-items-title">Recently Added</h3>
          <div className="mediaConnectActions">
            <span>{items.value?.page.returned ?? 0} shown</span>
            <Link
              className="mediaInlineLink"
              search={{ limit: 20, offset: 0 }}
              to="/media/items"
            >
              View all
            </Link>
          </div>
        </div>
        <MediaItemGrid
          emptyMessage="No recently added media"
          result={items}
        />
      </section>
    </section>
  );
}

export function MediaItemsPage({
  onSearchChange,
  search,
}: {
  onSearchChange: MediaSearchChange<MediaItemsBrowseSearch>;
  search: MediaItemsBrowseSearch;
}) {
  const { dataSource } = useMediaSession();
  const result = useMediaLoad(
    dataSource,
    (source) => loadMediaItems(source, search, MEDIA_ITEMS_ERROR),
    [search.facet, search.limit, search.offset, search.order, search.q, search.sort, search.watch_state],
  );
  const activeFilterCount = mediaItemsBrowseActiveFilterCount(search);
  const hasPaginationDelta = mediaItemsBrowseHasPaginationDelta(search);

  if (!dataSource) {
    return <MediaConnectPage />;
  }

  return (
    <section className="mediaPage" aria-labelledby="media-items-browse-title">
      <header className="mediaPageHeader">
        <div>
          <p className="mediaKicker">Browse</p>
          <h2 id="media-items-browse-title">Media Items</h2>
        </div>
        <span>{result.value?.page.returned ?? 0} shown</span>
      </header>
      <MediaItemsBrowseFilters
        activeFilterCount={activeFilterCount}
        hasPaginationDelta={hasPaginationDelta}
        onSearchChange={onSearchChange}
        search={search}
        searchEnabled
      />
      <MediaItemGrid emptyMessage="No media items" result={result} />
      <MediaPager
        label="Media Items"
        onSearchChange={onSearchChange}
        page={result.value?.page}
        search={search}
      />
    </section>
  );
}

function MediaItemsBrowseFilters({
  activeFilterCount,
  hasPaginationDelta,
  onSearchChange,
  search,
  searchEnabled,
}: {
  activeFilterCount: number;
  hasPaginationDelta: boolean;
  onSearchChange: MediaSearchChange<MediaItemsBrowseSearch>;
  search: MediaItemsBrowseSearch;
  searchEnabled?: boolean;
}) {
  return (
    <FilterBar label="Media item filters">
      {searchEnabled ? (
        <FilterField label="Search">
          <input
            aria-label="Search media items"
            placeholder="title or keyword"
            value={search.q ?? ""}
            onChange={(event) => onSearchChange(mediaItemsBrowseSearchChange(event.target.value))}
          />
        </FilterField>
      ) : null}
      <FilterField label="Facet">
        <input
          aria-label="Media item facet filter"
          placeholder="kind:movie"
          value={search.facet ?? ""}
          onChange={(event) => onSearchChange(mediaItemsBrowseFacetChange(event.target.value))}
        />
      </FilterField>
      <FilterField label="Sort">
        <select
          aria-label="Media item sort"
          value={search.sort ?? ""}
          onChange={(event) => onSearchChange(mediaItemsBrowseSortChange(event.target.value))}
        >
          <option value="">Default</option>
          <option value="date_added">Date added</option>
          <option value="title">Title</option>
          <option value="release_date">Release date</option>
          <option value="last_played">Last played</option>
        </select>
      </FilterField>
      <FilterField label="Order">
        <select
          aria-label="Media item sort order"
          value={search.order ?? ""}
          onChange={(event) => onSearchChange(mediaItemsBrowseOrderChange(event.target.value))}
        >
          <option value="">Default</option>
          <option value="desc">Descending</option>
          <option value="asc">Ascending</option>
        </select>
      </FilterField>
      <FilterField label="Watch state">
        <select
          aria-label="Media item watch state filter"
          value={search.watch_state ?? "any"}
          onChange={(event) => onSearchChange(mediaItemsBrowseWatchStateChange(event.target.value))}
        >
          <option value="any">Any</option>
          <option value="unwatched">Unwatched</option>
          <option value="in_progress">In progress</option>
          <option value="watched">Watched</option>
        </select>
      </FilterField>
      <FilterField label="Limit">
        <input
          aria-label="Media items page limit"
          min={1}
          type="number"
          value={search.limit}
          onChange={(event) => onSearchChange(mediaItemsBrowseLimitChange(event.target.value, search.limit))}
        />
      </FilterField>
      <FilterActions>
        <Button
          disabled={activeFilterCount === 0 && !hasPaginationDelta}
          onClick={() => onSearchChange(mediaItemsBrowseResetSearch())}
          variant="ghost"
        >
          <X size={16} />
          Clear
        </Button>
      </FilterActions>
    </FilterBar>
  );
}

export function MediaLibrariesPage({
  onSearchChange,
  search,
}: {
  onSearchChange: MediaSearchChange<MediaPageSearch>;
  search: MediaPageSearch;
}) {
  const { dataSource } = useMediaSession();
  const result = useMediaLoad(
    dataSource,
    (source) => source.listLibraries(search),
    [search.limit, search.offset],
  );

  if (!dataSource) {
    return <MediaConnectPage />;
  }

  return (
    <section className="mediaPage" aria-labelledby="media-libraries-title">
      <header className="mediaPageHeader">
        <div>
          <p className="mediaKicker">Browse</p>
          <h2 id="media-libraries-title">Media Libraries</h2>
        </div>
        <span>{result.value?.page.returned ?? 0} accessible</span>
      </header>
      <MediaLibraryGrid result={result} />
      <MediaPager
        label="Media Libraries"
        onSearchChange={onSearchChange}
        page={result.value?.page}
        search={search}
      />
    </section>
  );
}

export function MediaLibraryDetailPage({
  libraryId,
  onSearchChange,
  search,
}: {
  libraryId: string;
  onSearchChange: MediaSearchChange<MediaItemsBrowseSearch>;
  search: MediaItemsBrowseSearch;
}) {
  const { dataSource } = useMediaSession();
  const library = useMediaLoad(
    dataSource,
    (source) => source.getLibrary(libraryId),
    [libraryId],
  );
  const sources = useMediaLoad(
    dataSource,
    (source) =>
      source.listLibrarySources(libraryId, {
        limit: search.limit,
        offset: search.offset,
      }),
    [libraryId, search.limit, search.offset],
  );
  const items = useMediaLoad(
    dataSource,
    (source) => loadLibraryItems(source, libraryId, search, LIBRARY_ITEMS_ERROR),
    [libraryId, search.facet, search.limit, search.offset, search.order, search.sort, search.watch_state],
  );
  const activeFilterCount = mediaItemsBrowseActiveFilterCount(search);
  const hasPaginationDelta = mediaItemsBrowseHasPaginationDelta(search);

  if (!dataSource) {
    return <MediaConnectPage />;
  }

  return (
    <section className="mediaPage" aria-labelledby="media-library-title">
      {library.loading ? <div className="mediaSkeleton" /> : null}
      {library.error ? <div className="mediaError">{library.error}</div> : null}
      {library.value ? <MediaLibraryDetailHeader result={library.value} /> : null}
      <section className="mediaPanel" aria-labelledby="media-library-items-title">
        <div className="mediaPanelHeader">
          <h3 id="media-library-items-title">Library items</h3>
          <span>{items.value?.page.returned ?? 0} shown</span>
        </div>
        <MediaItemsBrowseFilters
          activeFilterCount={activeFilterCount}
          hasPaginationDelta={hasPaginationDelta}
          onSearchChange={onSearchChange}
          search={search}
        />
        <MediaItemGrid emptyMessage="No library items" result={items} />
        <MediaPager
          label="Library items"
          onSearchChange={onSearchChange}
          page={items.value?.page}
          search={search}
        />
      </section>
      <section className="mediaPanel" aria-labelledby="media-library-sources-title">
        <div className="mediaPanelHeader">
          <h3 id="media-library-sources-title">Library sources</h3>
          <span>{sources.value?.page.returned ?? 0} shown</span>
        </div>
        <MediaLibrarySources result={sources} />
        <MediaPager
          label="Library sources"
          onSearchChange={onSearchChange}
          page={sources.value?.page}
          search={search}
        />
      </section>
    </section>
  );
}

export function MediaSearchPage({
  onSearchChange,
  search,
}: {
  onSearchChange: MediaSearchChange<MediaSearchRouteSearch>;
  search: MediaSearchRouteSearch;
}) {
  const { dataSource } = useMediaSession();
  const [query, setQuery] = useState(search.q ?? "");
  const result = useMediaLoad(
    dataSource,
    (source) => loadMediaSearch(source, search, MEDIA_SEARCH_ERROR),
    [search.facet, search.limit, search.offset, search.q],
  );

  useEffect(() => {
    setQuery(search.q ?? "");
  }, [search.q]);

  if (!dataSource) {
    return <MediaConnectPage />;
  }

  return (
    <section className="mediaPage" aria-labelledby="media-search-title">
      <header className="mediaPageHeader">
        <div>
          <p className="mediaKicker">Find</p>
          <h2 id="media-search-title">Search</h2>
        </div>
        <span>{result.value?.page.returned ?? 0} results</span>
      </header>
      <form
        className="mediaSearch"
        onSubmit={(event) => {
          event.preventDefault();
          onSearchChange(mediaSearchSubmitChange(query));
        }}
      >
        <label>
          <span>Search media</span>
          <input
            onChange={(event) => setQuery(event.currentTarget.value)}
            value={query}
          />
        </label>
        <Button type="submit">
          <Search size={16} />
          <span>Search</span>
        </Button>
      </form>
      <MediaSearchResults result={result} />
      <MediaPager
        label="Search results"
        onSearchChange={onSearchChange}
        page={result.value?.page}
        search={search}
      />
    </section>
  );
}

function MediaContinueWatching({
  clearingItemId,
  error,
  onStartOver,
  result,
}: {
  clearingItemId: string | null;
  error: string | null;
  onStartOver(entry: ContinueWatchingResponse["items"][number]): void;
  result: MediaAsyncState<ContinueWatchingResponse>;
}) {
  return (
    <section className="mediaPanel" aria-labelledby="media-continue-title">
      <div className="mediaPanelHeader">
        <h3 id="media-continue-title">Continue Watching</h3>
        <span>{result.value?.page.returned ?? 0} active</span>
      </div>
      {result.loading ? <div className="mediaSkeleton" /> : null}
      {result.error ? <div className="mediaError">{result.error}</div> : null}
      {error ? <div className="mediaError">{error}</div> : null}
      {result.value?.items.length ? (
        <div className="mediaContinueList">
          {result.value.items.map((entry) => (
            <article className="mediaContinueRow" key={entry.item.id}>
              <div>
                <div className="mediaContinueContent">
                  <strong>{entry.item.metadata.title}</strong>
                  <span>
                    {Math.round((entry.state.progress_percent ?? 0) * 100)}% complete
                    {" - "}
                    resume at {formatRuntimeMs(entry.state.resume_position_ms)}
                  </span>
                </div>
                <Link
                  className="uiButton uiButtonDefault uiButtonSm"
                  params={{ itemId: entry.item.id }}
                  search={
                    entry.state.source_id
                      ? { source_id: entry.state.source_id }
                      : {}
                  }
                  to="/media/watch/$itemId"
                >
                  <Play size={15} />
                  <span>Resume</span>
                </Link>
                <Button
                  disabled={clearingItemId === entry.item.id}
                  onClick={() => onStartOver(entry)}
                  size="sm"
                  type="button"
                  variant="outline"
                >
                  <RotateCcw size={15} />
                  <span>Start over</span>
                </Button>
              </div>
              <progress value={entry.state.progress_percent ?? 0} max={1} />
            </article>
          ))}
        </div>
      ) : result.loading ? null : (
        <div className="mediaEmpty">No active playback state</div>
      )}
    </section>
  );
}

function MediaLibraryGrid({
  result,
}: {
  result: MediaAsyncState<LibraryListResponse>;
}) {
  if (result.loading) {
    return <div className="mediaSkeletonGrid" />;
  }

  if (result.error) {
    return <div className="mediaError">{result.error}</div>;
  }

  return (
    <div className="mediaLibraryGrid">
      {result.value?.libraries.map((library) => (
        <Link
          className="mediaLibraryCard"
          key={library.id}
          params={{ libraryId: library.id }}
          search={{ limit: 20, offset: 0 }}
          to="/media/libraries/$libraryId"
        >
          <span>{library.options.preset}</span>
          <strong>{library.name}</strong>
          <small>{library.options.metadata_profile.item_kinds.join(", ")}</small>
        </Link>
      ))}
    </div>
  );
}

function MediaLibraryDetailHeader({ result }: { result: LibraryResponse }) {
  const library = result.library;

  return (
    <header className="mediaPageHeader">
      <div>
        <p className="mediaKicker">Media Library</p>
        <h2 id="media-library-title">{library.name}</h2>
      </div>
      <div className="mediaMetaPills">
        <span>{library.options.preset}</span>
        <span>{library.options.domain}</span>
        <span>{library.options.metadata_profile.item_kinds.join(", ")}</span>
      </div>
    </header>
  );
}

function MediaLibrarySources({
  result,
}: {
  result: MediaAsyncState<LibrarySourcesResponse>;
}) {
  if (result.loading) {
    return <div className="mediaSkeletonGrid" />;
  }

  if (result.error) {
    return <div className="mediaError">{result.error}</div>;
  }

  if (!result.value?.sources.length) {
    return <div className="mediaEmpty">No accessible sources in this library</div>;
  }

  return (
    <div className="mediaSourceList">
      {result.value.sources.map((entry) => (
        <article className="mediaSourceRow" key={entry.source.id}>
          <div>
            <span>{entry.source.file_name}</span>
            {entry.item ? (
              <Link
                className="mediaInlineLink"
                to="/media/items/$itemId"
                params={{ itemId: entry.item.id }}
              >
                {entry.item.metadata.title}
              </Link>
            ) : (
              <strong>Unmatched source</strong>
            )}
          </div>
          <div className="mediaSourceFacts">
            <span>{formatBytes(entry.source.size_bytes)}</span>
            <span>{entry.probe?.container ?? "container unknown"}</span>
            <span>{formatRuntimeMs(entry.probe?.duration_ms)}</span>
          </div>
        </article>
      ))}
    </div>
  );
}

function MediaItemGrid({
  emptyMessage,
  result,
}: {
  emptyMessage: string;
  result: MediaAsyncState<ItemsResponse>;
}) {
  if (result.loading) {
    return <div className="mediaSkeletonGrid" />;
  }

  if (result.error) {
    return <div className="mediaError">{result.error}</div>;
  }

  if (!result.value?.items.length) {
    return <div className="mediaEmpty">{emptyMessage}</div>;
  }

  return (
    <div className="mediaItemGrid">
      {result.value?.items.map((item) => (
        <MediaItemCard item={item} key={item.id} />
      ))}
    </div>
  );
}

function MediaSearchResults({ result }: { result: MediaAsyncState<SearchResponse> }) {
  if (result.loading) {
    return <div className="mediaSkeletonGrid" />;
  }

  if (result.error) {
    return <div className="mediaError">{MEDIA_SEARCH_ERROR}</div>;
  }

  return (
    <div className="mediaItemGrid">
      {result.value?.hits.map((hit) => (
        <MediaItemCard
          badge={`${Math.round(hit.score * 100)} match`}
          item={hit.item}
          key={hit.item.id}
        />
      ))}
    </div>
  );
}

function MediaItemCard({ badge, item }: { badge?: string; item: MediaItemDto }) {
  return (
    <Link className="mediaItemCard" to="/media/items/$itemId" params={{ itemId: item.id }}>
      <span>{badge ?? item.kind}</span>
      <strong>{item.metadata.title}</strong>
      <small>{formatRuntimeMinutes(item.metadata.runtime_minutes)}</small>
    </Link>
  );
}

function MediaPager<TSearch extends MediaPageSearch>({
  label,
  onSearchChange,
  page,
  search,
}: {
  label: string;
  onSearchChange: MediaSearchChange<TSearch>;
  page?: { limit: number; offset: number; returned: number };
  search: TSearch;
}) {
  const canGoBack = search.offset > 0;
  const canGoForward = Boolean(page && page.returned >= search.limit);
  const rangeLabel =
    page?.returned && page.returned > 0
      ? `${search.offset + 1}-${search.offset + page.returned}`
      : "0-0";

  return (
    <div className="mediaPager" aria-label={`${label} pagination`}>
      <Button
        disabled={!canGoBack}
        onClick={() =>
          onSearchChange({ offset: Math.max(0, search.offset - search.limit) } as Partial<TSearch>)
        }
        size="sm"
        type="button"
        variant="outline"
      >
        <ArrowLeft size={15} />
        <span>Previous</span>
      </Button>
      <span>
        {rangeLabel}
      </span>
      <Button
        disabled={!canGoForward}
        onClick={() => onSearchChange({ offset: search.offset + search.limit } as Partial<TSearch>)}
        size="sm"
        type="button"
        variant="outline"
      >
        <span>Next</span>
        <ArrowRight size={15} />
      </Button>
    </div>
  );
}
