import { Link } from "@tanstack/react-router";
import type {
  ContinueWatchingResponse,
  ItemDetailResponse,
  ItemsResponse,
  LibraryListResponse,
  LibraryResponse,
  LibrarySourcesResponse,
  MediaItemDto,
  MediaSourceDto,
  SearchResponse,
} from "@nako/sdk";
import { ArrowLeft, ArrowRight, Search } from "lucide-react";
import { useEffect, useState, type FormEvent } from "react";

import { Button } from "../../components/ui/Button";
import type {
  MediaConnection,
  MediaLoadResult,
  MediaWebDataSource,
} from "./mediaDataSource";
import { useMediaSession } from "./MediaSession";

export type MediaPageSearch = {
  limit: number;
  offset: number;
};

export type MediaSearchRouteSearch = MediaPageSearch & {
  facet?: string;
  q?: string;
};

type MediaSearchChange<TSearch> = (next: Partial<TSearch>) => void;

export function MediaHomePage() {
  const { dataSource } = useMediaSession();
  const continueWatching = useMediaLoad(dataSource, (source) => source.listContinueWatching());
  const items = useMediaLoad(dataSource, (source) => source.listItems({ limit: 8, offset: 0 }));

  if (!dataSource) {
    return <MediaConnectPage />;
  }

  return (
    <section className="mediaPage" aria-labelledby="media-home-title">
      <header className="mediaPageHeader">
        <h2 id="media-home-title">Watch next</h2>
      </header>
      <MediaContinueWatching result={continueWatching} />
      <section className="mediaPanel" aria-labelledby="media-items-title">
        <div className="mediaPanelHeader">
          <h3 id="media-items-title">Media Items</h3>
          <span>{items.value?.page.returned ?? 0} shown</span>
        </div>
        <MediaItemGrid result={items} />
      </section>
    </section>
  );
}

export function MediaConnectPage() {
  const { connect, connectionError, connecting } = useMediaSession();
  const [baseUrl, setBaseUrl] = useState("http://127.0.0.1:3000");
  const [bearerToken, setBearerToken] = useState("");

  async function submitLiveConnection(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const connection: MediaConnection = {
      mode: "live",
      baseUrl: baseUrl.trim(),
      bearerToken,
    };
    await connect(connection);
  }

  return (
    <section className="mediaConnect" aria-labelledby="media-connect-title">
      <div className="mediaConnectIntro">
        <p className="mediaKicker">Connect</p>
        <h2 id="media-connect-title">Enter a Nako server</h2>
        <p>
          The token stays in memory for this browser session. Fixture mode uses
          local development data.
        </p>
      </div>
      <form className="mediaConnectForm" onSubmit={submitLiveConnection}>
        <label>
          <span>Server URL</span>
          <input
            autoComplete="url"
            onChange={(event) => setBaseUrl(event.currentTarget.value)}
            required
            type="url"
            value={baseUrl}
          />
        </label>
        <label>
          <span>Access token</span>
          <input
            autoComplete="off"
            onChange={(event) => setBearerToken(event.currentTarget.value)}
            required
            type="password"
            value={bearerToken}
          />
        </label>
        {connectionError ? <div className="mediaError">{connectionError}</div> : null}
        <div className="mediaConnectActions">
          <Button disabled={connecting} type="submit">
            {connecting ? "Connecting" : "Connect"}
          </Button>
          <Button onClick={() => void connect({ mode: "fixture" })} type="button" variant="outline">
            Use fixture demo
          </Button>
        </div>
      </form>
    </section>
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
  onSearchChange: MediaSearchChange<MediaPageSearch>;
  search: MediaPageSearch;
}) {
  const { dataSource } = useMediaSession();
  const library = useMediaLoad(
    dataSource,
    (source) => source.getLibrary(libraryId),
    [libraryId],
  );
  const sources = useMediaLoad(
    dataSource,
    (source) => source.listLibrarySources(libraryId, search),
    [libraryId, search.limit, search.offset],
  );

  if (!dataSource) {
    return <MediaConnectPage />;
  }

  return (
    <section className="mediaPage" aria-labelledby="media-library-title">
      {library.loading ? <div className="mediaSkeleton" /> : null}
      {library.error ? <div className="mediaError">{library.error}</div> : null}
      {library.value ? <MediaLibraryDetailHeader result={library.value} /> : null}
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
    (source) =>
      source.searchItems({
        facet: search.facet,
        limit: search.limit,
        offset: search.offset,
        q: search.q,
      }),
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
          onSearchChange({ offset: 0, q: query.trim() || undefined });
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

export function MediaItemDetailPage({ itemId }: { itemId: string }) {
  const { dataSource } = useMediaSession();
  const result = useMediaLoad(dataSource, (source) => source.getItem(itemId), [itemId]);

  if (!dataSource) {
    return <MediaConnectPage />;
  }

  if (result.loading) {
    return <div className="mediaSkeletonGrid" />;
  }

  if (result.error) {
    return <div className="mediaError">{result.error}</div>;
  }

  if (!result.value) {
    return <div className="mediaEmpty">Media Item unavailable</div>;
  }

  return <MediaItemDetail result={result.value} />;
}

function MediaContinueWatching({
  result,
}: {
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
      {result.value?.items.length ? (
        <div className="mediaContinueList">
          {result.value.items.map((entry) => (
            <article className="mediaContinueRow" key={entry.item.id}>
              <div>
                <strong>{entry.item.metadata.title}</strong>
                <span>{Math.round((entry.state.progress_percent ?? 0) * 100)}% complete</span>
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

function MediaItemGrid({ result }: { result: MediaAsyncState<ItemsResponse> }) {
  if (result.loading) {
    return <div className="mediaSkeletonGrid" />;
  }

  if (result.error) {
    return <div className="mediaError">{result.error}</div>;
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
    return <div className="mediaError">{result.error}</div>;
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

function MediaItemDetail({ result }: { result: ItemDetailResponse }) {
  const metadata = result.item.metadata;

  return (
    <section className="mediaPage" aria-labelledby="media-item-title">
      <header className="mediaItemHero">
        <div>
          <p className="mediaKicker">{result.item.kind}</p>
          <h2 id="media-item-title">{metadata.title}</h2>
          {metadata.overview ? <p>{metadata.overview}</p> : null}
        </div>
        <div className="mediaMetaPills">
          <span>{formatRuntimeMinutes(metadata.runtime_minutes)}</span>
          {metadata.release_date ? <span>{metadata.release_date}</span> : null}
          {metadata.genres.slice(0, 3).map((genre) => (
            <span key={genre}>{genre}</span>
          ))}
        </div>
      </header>
      <section className="mediaPanel" aria-labelledby="media-item-sources-title">
        <div className="mediaPanelHeader">
          <h3 id="media-item-sources-title">Available sources</h3>
          <span>{result.sources.length} versions</span>
        </div>
        <div className="mediaSourceList">
          {result.sources.map((source) => (
            <article className="mediaSourceRow" key={source.id}>
              <div>
                <span>{source.file_name}</span>
                <strong>{sourceSummary(source)}</strong>
              </div>
              <div className="mediaSourceFacts">
                <span>{formatBytes(source.size_bytes)}</span>
                <span>{source.library_id}</span>
              </div>
            </article>
          ))}
        </div>
      </section>
      <section className="mediaPanel" aria-labelledby="media-item-metadata-title">
        <div className="mediaPanelHeader">
          <h3 id="media-item-metadata-title">Metadata</h3>
        </div>
        <div className="mediaFactGrid">
          <div>
            <span>Original title</span>
            <strong>{metadata.original_title ?? "Unavailable"}</strong>
          </div>
          <div>
            <span>Studios</span>
            <strong>{metadata.studios.map((studio) => studio.name).join(", ") || "Unavailable"}</strong>
          </div>
          <div>
            <span>Tags</span>
            <strong>{metadata.tags.join(", ") || "Unavailable"}</strong>
          </div>
        </div>
      </section>
    </section>
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

  return (
    <div className="mediaPager" aria-label={`${label} pagination`}>
      <Button
        disabled={!canGoBack}
        onClick={() => onSearchChange({ offset: Math.max(0, search.offset - search.limit) } as Partial<TSearch>)}
        size="sm"
        type="button"
        variant="outline"
      >
        <ArrowLeft size={15} />
        <span>Previous</span>
      </Button>
      <span>
        {search.offset + 1}-{search.offset + (page?.returned ?? 0)}
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

type MediaAsyncState<T> = {
  error: string | null;
  loading: boolean;
  value: T | null;
};

function useMediaLoad<T>(
  dataSource: MediaWebDataSource | null,
  load: (dataSource: MediaWebDataSource) => Promise<MediaLoadResult<T>>,
  deps: readonly unknown[] = [],
): MediaAsyncState<T> {
  const [state, setState] = useState<MediaAsyncState<T>>({
    error: null,
    loading: Boolean(dataSource),
    value: null,
  });

  useEffect(() => {
    let cancelled = false;
    if (!dataSource) {
      setState({ error: null, loading: false, value: null });
      return;
    }

    setState((current) => ({ ...current, error: null, loading: true }));
    load(dataSource)
      .then((result) => {
        if (!cancelled) {
          setState({ error: result.error ?? null, loading: false, value: result.value });
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setState({
            error: error instanceof Error ? error.message : "Media request failed",
            loading: false,
            value: null,
          });
        }
      });

    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [dataSource, ...deps]);

  return state;
}

function formatBytes(value: number | null) {
  if (!value) {
    return "size unknown";
  }

  const gib = value / 1024 / 1024 / 1024;
  return `${gib.toFixed(gib >= 10 ? 0 : 1)} GiB`;
}

function formatRuntimeMinutes(value: number | null) {
  return value ? `${value} min` : "Runtime unknown";
}

function formatRuntimeMs(value: number | null | undefined) {
  return value ? `${Math.round(value / 60_000)} min` : "duration unknown";
}

function sourceSummary(source: MediaSourceDto) {
  return source.size_bytes ? "Local source" : "Source";
}
