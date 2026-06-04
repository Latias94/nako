import { Link } from "@tanstack/react-router";
import type {
  BrowserPlaybackTicketRequest,
  BrowserPlaybackTicketResponse,
  ContinueWatchingResponse,
  ItemDetailResponse,
  ItemsResponse,
  LibraryListResponse,
  LibraryResponse,
  LibrarySourcesResponse,
  MediaItemDto,
  MediaSourceDto,
  PlaybackCapabilitiesQuery,
  PlaybackDecisionResponse,
  SearchResponse,
  UserPlaybackStateResponse,
} from "@nako/sdk";
import { ArrowLeft, ArrowRight, RefreshCw, Search } from "lucide-react";
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type FormEvent,
  type RefObject,
} from "react";

import { Button } from "../../components/ui/Button";
import type {
  MediaConnection,
  MediaLoadResult,
  MediaWebDataSource,
} from "./mediaDataSource";
import { useMediaSession } from "./MediaSession";

const MEDIA_PROGRESS_WRITE_INTERVAL_MS = 30_000;

export type MediaPageSearch = {
  limit: number;
  offset: number;
};

export type MediaSearchRouteSearch = MediaPageSearch & {
  facet?: string;
  q?: string;
};

export type MediaItemSearch = {
  source_id?: string;
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

type MediaItemPageProps = {
  itemId: string;
  onSearchChange: MediaSearchChange<MediaItemSearch>;
  search: MediaItemSearch;
};

export function MediaItemDetailPage(props: MediaItemPageProps) {
  const playback = useMediaItemPlayback(props);

  if (!playback.dataSource) {
    return <MediaConnectPage />;
  }

  if (playback.result.loading) {
    return <div className="mediaSkeletonGrid" />;
  }

  if (playback.result.error) {
    return <div className="mediaError">{playback.result.error}</div>;
  }

  if (!playback.result.value) {
    return <div className="mediaEmpty">Media Item unavailable</div>;
  }

  return (
    <MediaItemDetail
      decision={playback.decision}
      mutationError={playback.mutationError}
      onMarkWatched={playback.onMarkWatched}
      onSourceChange={playback.onSourceChange}
      playbackState={playback.playbackState}
      result={playback.result.value}
      savingPlaybackState={playback.savingPlaybackState}
      selectedSourceId={playback.selectedSourceId}
    />
  );
}

export function MediaWatchPage(props: MediaItemPageProps) {
  const playback = useMediaItemPlayback(props);

  if (!playback.dataSource) {
    return <MediaConnectPage />;
  }

  if (playback.result.loading) {
    return <div className="mediaSkeletonGrid" />;
  }

  if (playback.result.error) {
    return <div className="mediaError">{playback.result.error}</div>;
  }

  if (!playback.result.value) {
    return <div className="mediaEmpty">Media Item unavailable</div>;
  }

  return (
    <MediaWatch
      browserTicket={playback.browserTicket}
      decision={playback.decision}
      mutationError={playback.mutationError}
      onMarkWatched={playback.onMarkWatched}
      onBrowserTicketRetry={playback.onBrowserTicketRetry}
      onPlaybackEnded={playback.onPlaybackEnded}
      onPlaybackPaused={playback.onPlaybackPaused}
      onPlaybackProgress={playback.onPlaybackProgress}
      onPlaybackStarted={playback.onPlaybackStarted}
      onSourceChange={playback.onSourceChange}
      playbackState={playback.playbackState}
      result={playback.result.value}
      savingPlaybackState={playback.savingPlaybackState}
      selectedSourceId={playback.selectedSourceId}
    />
  );
}

function useMediaItemPlayback({
  itemId,
  onSearchChange,
  search,
}: MediaItemPageProps) {
  const { dataSource } = useMediaSession();
  const browserCapabilities = useMemo(() => detectBrowserPlaybackCapabilities(), []);
  const result = useMediaLoad(dataSource, (source) => source.getItem(itemId), [itemId]);
  const selectedSourceId = search.source_id ?? result.value?.sources[0]?.id;
  const decision = useMediaLoad(
    selectedSourceId ? dataSource : null,
    (source) => source.getPlaybackDecision(
      selectedSourceId!,
      playbackCapabilitiesQuery(browserCapabilities),
    ),
    [selectedSourceId, browserCapabilities],
  );
  const playbackState = useMediaLoad(
    dataSource,
    (source) => source.getUserPlaybackState(itemId),
    [itemId],
  );
  const [browserTicketRetryKey, setBrowserTicketRetryKey] = useState(0);
  const browserTicket = useMediaLoad(
    selectedSourceId && decision.value ? dataSource : null,
    (source) =>
      source.createBrowserPlaybackTicket(
        selectedSourceId!,
        browserPlaybackTicketRequest(decision.value!, browserCapabilities),
      ),
    [
      selectedSourceId,
      decision.value?.decision.mode,
      decision.value?.decision.transcode_plan?.output_container,
      browserCapabilities,
      browserTicketRetryKey,
    ],
  );
  const [playbackStateOverride, setPlaybackStateOverride] =
    useState<UserPlaybackStateResponse | null>(null);
  const [playbackMutationError, setPlaybackMutationError] = useState<string | null>(null);
  const [savingPlaybackState, setSavingPlaybackState] = useState(false);
  const fallbackDurationMs = playbackDurationMs(result.value, decision.value);
  const playbackProgress = useMediaPlaybackProgress({
    dataSource,
    fallbackDurationMs,
    itemId,
    selectedSourceId,
    setPlaybackMutationError,
    setPlaybackStateOverride,
  });

  async function markWatched(watched: boolean) {
    if (!dataSource || !selectedSourceId) {
      return;
    }

    setSavingPlaybackState(true);
    setPlaybackMutationError(null);
    try {
      const response = await dataSource.setUserWatchedState(itemId, {
        duration_ms: fallbackDurationMs,
        position_ms: watched ? fallbackDurationMs : playbackState.value?.state.resume_position_ms,
        source_id: selectedSourceId,
        watched,
      });
      setPlaybackStateOverride(response.value);
    } catch (error: unknown) {
      setPlaybackMutationError(
        error instanceof Error ? error.message : "Playback state update failed",
      );
    } finally {
      setSavingPlaybackState(false);
    }
  }

  function selectSource(sourceId: string) {
    onSearchChange({ source_id: sourceId });
    setBrowserTicketRetryKey(0);
    setPlaybackStateOverride(null);
    setPlaybackMutationError(null);
  }

  return {
    dataSource,
    browserTicket,
    decision,
    mutationError: playbackMutationError,
    onMarkWatched: markWatched,
    onBrowserTicketRetry: () => setBrowserTicketRetryKey((current) => current + 1),
    onPlaybackEnded: playbackProgress.onEnded,
    onPlaybackPaused: playbackProgress.onPaused,
    onPlaybackProgress: playbackProgress.onProgress,
    onPlaybackStarted: playbackProgress.onStarted,
    onSourceChange: selectSource,
    playbackState: {
      ...playbackState,
      value: playbackStateOverride ?? playbackState.value,
    },
    result,
    savingPlaybackState,
    selectedSourceId,
  };
}

type MediaPlaybackProgressSnapshot = {
  durationMs: number | null;
  positionMs: number;
};

function useMediaPlaybackProgress({
  dataSource,
  fallbackDurationMs,
  itemId,
  selectedSourceId,
  setPlaybackMutationError,
  setPlaybackStateOverride,
}: {
  dataSource: MediaWebDataSource | null;
  fallbackDurationMs: number | null;
  itemId: string;
  selectedSourceId: string | undefined;
  setPlaybackMutationError(value: string | null): void;
  setPlaybackStateOverride(value: UserPlaybackStateResponse | null): void;
}) {
  const playbackStartedRef = useRef(false);
  const lastProgressPositionRef = useRef<number | null>(null);

  useEffect(() => {
    playbackStartedRef.current = false;
    lastProgressPositionRef.current = null;
  }, [dataSource, itemId, selectedSourceId]);

  const onStarted = useCallback(() => {
    playbackStartedRef.current = true;
  }, []);

  const writeProgress = useCallback(
    async (snapshot: MediaPlaybackProgressSnapshot, force: boolean) => {
      if (
        !dataSource ||
        !selectedSourceId ||
        !playbackStartedRef.current ||
        snapshot.positionMs <= 0
      ) {
        return;
      }

      const lastPositionMs = lastProgressPositionRef.current;
      if (lastPositionMs === snapshot.positionMs) {
        return;
      }
      if (!force) {
        const positionDeltaMs =
          lastPositionMs === null
            ? snapshot.positionMs
            : Math.abs(snapshot.positionMs - lastPositionMs);
        if (positionDeltaMs < MEDIA_PROGRESS_WRITE_INTERVAL_MS) {
          return;
        }
      }

      lastProgressPositionRef.current = snapshot.positionMs;
      try {
        const response = await dataSource.updateUserPlaybackProgress(itemId, {
          duration_ms: snapshot.durationMs ?? fallbackDurationMs,
          position_ms: snapshot.positionMs,
          source_id: selectedSourceId,
        });
        setPlaybackMutationError(null);
        setPlaybackStateOverride(response.value);
      } catch (error: unknown) {
        setPlaybackMutationError(
          error instanceof Error ? error.message : "Playback progress update failed",
        );
      }
    },
    [
      dataSource,
      fallbackDurationMs,
      itemId,
      selectedSourceId,
      setPlaybackMutationError,
      setPlaybackStateOverride,
    ],
  );

  const markEndedWatched = useCallback(
    async (snapshot: MediaPlaybackProgressSnapshot) => {
      if (!dataSource || !selectedSourceId || !playbackStartedRef.current) {
        return;
      }

      const durationMs = snapshot.durationMs ?? fallbackDurationMs;
      const positionMs = durationMs ?? snapshot.positionMs;
      if (positionMs <= 0) {
        return;
      }

      lastProgressPositionRef.current = positionMs;
      try {
        const response = await dataSource.setUserWatchedState(itemId, {
          duration_ms: durationMs,
          position_ms: positionMs,
          source_id: selectedSourceId,
          watched: true,
        });
        setPlaybackMutationError(null);
        setPlaybackStateOverride(response.value);
      } catch (error: unknown) {
        setPlaybackMutationError(
          error instanceof Error ? error.message : "Playback watched update failed",
        );
      }
    },
    [
      dataSource,
      fallbackDurationMs,
      itemId,
      selectedSourceId,
      setPlaybackMutationError,
      setPlaybackStateOverride,
    ],
  );

  return {
    onEnded: (snapshot: MediaPlaybackProgressSnapshot) => {
      void markEndedWatched(snapshot);
    },
    onPaused: (snapshot: MediaPlaybackProgressSnapshot) => {
      void writeProgress(snapshot, true);
    },
    onProgress: (snapshot: MediaPlaybackProgressSnapshot) => {
      void writeProgress(snapshot, false);
    },
    onStarted,
  };
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

function MediaItemDetail({
  decision,
  mutationError,
  onMarkWatched,
  onSourceChange,
  playbackState,
  result,
  savingPlaybackState,
  selectedSourceId,
}: {
  decision: MediaAsyncState<PlaybackDecisionResponse>;
  mutationError: string | null;
  onMarkWatched(watched: boolean): void;
  onSourceChange(sourceId: string): void;
  playbackState: MediaAsyncState<UserPlaybackStateResponse>;
  result: ItemDetailResponse;
  savingPlaybackState: boolean;
  selectedSourceId: string | undefined;
}) {
  const metadata = result.item.metadata;
  const selectedSource =
    result.sources.find((source) => source.id === selectedSourceId) ?? result.sources[0];

  return (
    <section className="mediaPage" aria-labelledby="media-item-title">
      <header className="mediaItemHero">
        <div>
          <p className="mediaKicker">{result.item.kind}</p>
          <h2 id="media-item-title">{metadata.title}</h2>
          {metadata.overview ? <p>{metadata.overview}</p> : null}
        </div>
        <div className="mediaHeroActions">
          <div className="mediaMetaPills">
            <span>{formatRuntimeMinutes(metadata.runtime_minutes)}</span>
            {metadata.release_date ? <span>{metadata.release_date}</span> : null}
            {metadata.genres.slice(0, 3).map((genre) => (
              <span key={genre}>{genre}</span>
            ))}
          </div>
          <Link
            className="uiButton uiButtonDefault uiButtonSm"
            params={{ itemId: result.item.id }}
            search={selectedSource ? { source_id: selectedSource.id } : {}}
            to="/media/watch/$itemId"
          >
            Watch
          </Link>
        </div>
      </header>
      <MediaSourceVersions
        onSourceChange={onSourceChange}
        result={result}
        selectedSource={selectedSource}
      />
      <section className="mediaPanel" aria-labelledby="media-playback-decision-title">
        <div className="mediaPanelHeader">
          <h3 id="media-playback-decision-title">Playback decision</h3>
          <span>{decision.value?.decision.mode ?? "pending"}</span>
        </div>
        <MediaPlaybackDecision result={decision} />
      </section>
      <section className="mediaPanel" aria-labelledby="media-playback-state-title">
        <div className="mediaPanelHeader">
          <h3 id="media-playback-state-title">Playback state</h3>
          <span>{playbackState.value?.state.watched ? "watched" : "in progress"}</span>
        </div>
        <MediaPlaybackState
          disabled={savingPlaybackState}
          error={mutationError}
          onMarkWatched={onMarkWatched}
          result={playbackState}
          selectedSource={selectedSource}
        />
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

function MediaWatch({
  browserTicket,
  decision,
  mutationError,
  onMarkWatched,
  onBrowserTicketRetry,
  onPlaybackEnded,
  onPlaybackPaused,
  onPlaybackProgress,
  onPlaybackStarted,
  onSourceChange,
  playbackState,
  result,
  savingPlaybackState,
  selectedSourceId,
}: {
  browserTicket: MediaAsyncState<BrowserPlaybackTicketResponse>;
  decision: MediaAsyncState<PlaybackDecisionResponse>;
  mutationError: string | null;
  onMarkWatched(watched: boolean): void;
  onBrowserTicketRetry(): void;
  onPlaybackEnded(snapshot: MediaPlaybackProgressSnapshot): void;
  onPlaybackPaused(snapshot: MediaPlaybackProgressSnapshot): void;
  onPlaybackProgress(snapshot: MediaPlaybackProgressSnapshot): void;
  onPlaybackStarted(): void;
  onSourceChange(sourceId: string): void;
  playbackState: MediaAsyncState<UserPlaybackStateResponse>;
  result: ItemDetailResponse;
  savingPlaybackState: boolean;
  selectedSourceId: string | undefined;
}) {
  const metadata = result.item.metadata;
  const selectedSource =
    result.sources.find((source) => source.id === selectedSourceId) ?? result.sources[0];
  const fallbackDurationMs = playbackDurationMs(result, decision.value);

  return (
    <section className="mediaPage" aria-labelledby="media-watch-title">
      <header className="mediaItemHero">
        <div>
          <p className="mediaKicker">Playback</p>
          <h2 id="media-watch-title">{metadata.title}</h2>
          <p>{selectedSource?.file_name ?? "No source selected"}</p>
        </div>
        <div className="mediaMetaPills">
          <span>{decision.value?.decision.mode ?? "decision pending"}</span>
          <span>{formatRuntimeMinutes(metadata.runtime_minutes)}</span>
        </div>
      </header>
      <section className="mediaPanel mediaPlayerShell" aria-labelledby="media-player-title">
        <div className="mediaPanelHeader">
          <h3 id="media-player-title">Player</h3>
          <span>{browserTicket.value?.mode ?? decision.value?.decision.mode ?? "pending"}</span>
        </div>
        <MediaBrowserPlayer
          fallbackDurationMs={fallbackDurationMs}
          onBrowserTicketRetry={onBrowserTicketRetry}
          onPlaybackEnded={onPlaybackEnded}
          onPlaybackPaused={onPlaybackPaused}
          onPlaybackProgress={onPlaybackProgress}
          onPlaybackStarted={onPlaybackStarted}
          result={browserTicket}
          title={metadata.title}
        />
      </section>
      <MediaSourceVersions
        onSourceChange={onSourceChange}
        result={result}
        selectedSource={selectedSource}
      />
      <section className="mediaPanel" aria-labelledby="media-playback-decision-title">
        <div className="mediaPanelHeader">
          <h3 id="media-playback-decision-title">Playback decision</h3>
          <span>{decision.value?.decision.mode ?? "pending"}</span>
        </div>
        <MediaPlaybackDecision result={decision} />
      </section>
      <section className="mediaPanel" aria-labelledby="media-playback-state-title">
        <div className="mediaPanelHeader">
          <h3 id="media-playback-state-title">Playback state</h3>
          <span>{playbackState.value?.state.watched ? "watched" : "in progress"}</span>
        </div>
        <MediaPlaybackState
          disabled={savingPlaybackState}
          error={mutationError}
          onMarkWatched={onMarkWatched}
          result={playbackState}
          selectedSource={selectedSource}
        />
      </section>
    </section>
  );
}

function MediaSourceVersions({
  onSourceChange,
  result,
  selectedSource,
}: {
  onSourceChange(sourceId: string): void;
  result: ItemDetailResponse;
  selectedSource: MediaSourceDto | undefined;
}) {
  return (
    <section className="mediaPanel" aria-labelledby="media-item-sources-title">
      <div className="mediaPanelHeader">
        <h3 id="media-item-sources-title">Source versions</h3>
        <span>{result.sources.length} versions</span>
      </div>
      <div className="mediaSectionHint">Available sources</div>
      <div className="mediaSourceList">
        {result.sources.map((source) => (
          <button
            aria-pressed={source.id === selectedSource?.id}
            className={
              source.id === selectedSource?.id
                ? "mediaSourceRow mediaSourceButton active"
                : "mediaSourceRow mediaSourceButton"
            }
            key={source.id}
            onClick={() => onSourceChange(source.id)}
            type="button"
          >
            <div>
              <span>{source.file_name}</span>
              <strong>{sourceSummary(source)}</strong>
            </div>
            <div className="mediaSourceFacts">
              <span>{formatBytes(source.size_bytes)}</span>
              <span>{source.library_id}</span>
            </div>
          </button>
        ))}
      </div>
    </section>
  );
}

function MediaPlaybackDecision({
  result,
}: {
  result: MediaAsyncState<PlaybackDecisionResponse>;
}) {
  if (result.loading) {
    return <div className="mediaSkeleton" />;
  }

  if (result.error) {
    return <div className="mediaError">{result.error}</div>;
  }

  if (!result.value) {
    return <div className="mediaEmpty">No playback decision available</div>;
  }

  const decision = result.value.decision;

  return (
    <div className="mediaDecisionGrid">
      <div>
        <span>Mode</span>
        <strong>{decision.mode}</strong>
      </div>
      <div>
        <span>Reason</span>
        <strong>{decision.reason}</strong>
      </div>
      <div>
        <span>Container</span>
        <strong>{result.value.probe?.container ?? "unknown"}</strong>
      </div>
      <div>
        <span>Range</span>
        <strong>
          {decision.direct_play?.supports_range_requests ? "range ready" : "range unknown"}
        </strong>
      </div>
      <div>
        <span>Transport</span>
        <strong>Issued on watch page</strong>
      </div>
    </div>
  );
}

function MediaBrowserPlayer({
  fallbackDurationMs,
  onBrowserTicketRetry,
  onPlaybackEnded,
  onPlaybackPaused,
  onPlaybackProgress,
  onPlaybackStarted,
  result,
  title,
}: {
  fallbackDurationMs: number | null;
  onBrowserTicketRetry(): void;
  onPlaybackEnded(snapshot: MediaPlaybackProgressSnapshot): void;
  onPlaybackPaused(snapshot: MediaPlaybackProgressSnapshot): void;
  onPlaybackProgress(snapshot: MediaPlaybackProgressSnapshot): void;
  onPlaybackStarted(): void;
  result: MediaAsyncState<BrowserPlaybackTicketResponse>;
  title: string;
}) {
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const [candidateSelectionState, setCandidateSelectionState] =
    useState<MediaPlaybackCandidateSelection>({
      activeCandidateIndex: 0,
      failedCandidateKey: null,
      retryCount: 0,
      ticketSignature: null,
    });
  const ticketSignature = result.value ? playbackTicketSignature(result.value) : null;
  const candidateSelection = mediaPlaybackCandidateSelectionFor(
    candidateSelectionState,
    ticketSignature,
  );

  if (result.loading) {
    return <div className="mediaSkeleton" />;
  }

  if (result.error) {
    return (
      <div className="mediaError">
        <span>Playback ticket could not be issued. Request a fresh ticket and try again.</span>
        <Button
          onClick={onBrowserTicketRetry}
          size="sm"
          type="button"
          variant="outline"
        >
          <RefreshCw size={15} />
          <span>Retry ticket</span>
        </Button>
      </div>
    );
  }

  const ticket = result.value;
  const candidates = ticket ? playbackCandidates(ticket) : [];
  const activeCandidate =
    candidates[
      Math.min(candidateSelection.activeCandidateIndex, Math.max(0, candidates.length - 1))
    ];

  if (!ticket || !activeCandidate) {
    return (
      <div className="mediaEmpty">
        <span>Playback URL unavailable</span>
        <Button
          onClick={onBrowserTicketRetry}
          size="sm"
          type="button"
          variant="outline"
        >
          <RefreshCw size={15} />
          <span>Retry ticket</span>
        </Button>
      </div>
    );
  }

  const adapter = playbackAdapterFor(activeCandidate);
  const playerFailed = candidateSelection.failedCandidateKey === activeCandidate.key;
  const nextCandidate = nextPlaybackCandidate(
    candidates,
    candidateSelection.activeCandidateIndex,
  );
  const attachHlsJs = adapter.kind === "hls-js";
  const markCandidateFailed = () => {
    setCandidateSelectionState((current) => ({
      ...mediaPlaybackCandidateSelectionFor(current, ticketSignature),
      failedCandidateKey: activeCandidate.key,
    }));
  };
  const retryActiveCandidate = () => {
    setCandidateSelectionState((current) => {
      const selection = mediaPlaybackCandidateSelectionFor(current, ticketSignature);
      return {
        ...selection,
        failedCandidateKey: null,
        retryCount: selection.retryCount + 1,
      };
    });
  };
  const switchToCandidate = (candidate: MediaPlaybackCandidate) => {
    setCandidateSelectionState((current) => {
      const selection = mediaPlaybackCandidateSelectionFor(current, ticketSignature);
      return {
        ...selection,
        activeCandidateIndex: candidate.index,
        failedCandidateKey: null,
        retryCount: selection.retryCount + 1,
      };
    });
  };

  return (
    <div className="mediaPlayerFrame">
      {adapter.kind === "unsupported-hls" ? (
        <div className="mediaError">
          <span>
            This browser cannot open the HLS playlist without a compatible playback adapter.
          </span>
          {nextCandidate ? (
            <Button
              onClick={() => switchToCandidate(nextCandidate)}
              size="sm"
              type="button"
              variant="outline"
            >
              <ArrowRight size={15} />
              <span>Try next path</span>
            </Button>
          ) : null}
          <Button
            onClick={onBrowserTicketRetry}
            size="sm"
            type="button"
            variant="outline"
          >
            <RefreshCw size={15} />
            <span>Retry ticket</span>
          </Button>
        </div>
      ) : (
        <MediaVideoElement
          adapter={adapter}
          attachHlsJs={attachHlsJs}
          candidate={activeCandidate}
          fallbackDurationMs={fallbackDurationMs}
          onFailure={markCandidateFailed}
          onPlaybackEnded={onPlaybackEnded}
          onPlaybackPaused={onPlaybackPaused}
          onPlaybackProgress={onPlaybackProgress}
          onPlaybackStarted={onPlaybackStarted}
          retryCount={candidateSelection.retryCount}
          title={title}
          videoRef={videoRef}
        />
      )}
      {playerFailed ? (
        <div className="mediaError">
          <span>Playback failed before the browser could start the stream.</span>
          {nextCandidate ? (
            <Button
              onClick={() => switchToCandidate(nextCandidate)}
              size="sm"
              type="button"
              variant="outline"
            >
              <ArrowRight size={15} />
              <span>Try next path</span>
            </Button>
          ) : null}
          <Button
            onClick={retryActiveCandidate}
            size="sm"
            type="button"
            variant="outline"
          >
            <RefreshCw size={15} />
            <span>Retry playback</span>
          </Button>
        </div>
      ) : null}
      <div className="mediaPlayerFacts">
        <span>{ticket.mode}</span>
        <span>{activeCandidate.contentType}</span>
        <span>{adapter.label}</span>
        <span>{activeCandidate.supportsRangeRequests ? "range ready" : "playlist"}</span>
        <span>expires {ticket.expires_at}</span>
      </div>
    </div>
  );
}

type MediaPlaybackCandidate = {
  contentType: string;
  index: number;
  key: string;
  kind: string;
  supportsRangeRequests: boolean;
  url: string;
};

type MediaPlaybackCandidateSelection = {
  activeCandidateIndex: number;
  failedCandidateKey: string | null;
  retryCount: number;
  ticketSignature: string | null;
};

type MediaPlaybackAdapter = {
  kind: "native-video" | "native-hls" | "hls-js" | "unsupported-hls";
  label: string;
};

type HlsJsInstance = {
  attachMedia(video: HTMLVideoElement): void;
  destroy(): void;
  loadSource(url: string): void;
  on?(event: string, handler: () => void): void;
};

type HlsJsConstructor = {
  Events?: {
    ERROR?: string;
  };
  isSupported?: () => boolean;
  new (): HlsJsInstance;
};

function MediaVideoElement({
  adapter,
  attachHlsJs,
  candidate,
  fallbackDurationMs,
  onFailure,
  onPlaybackEnded,
  onPlaybackPaused,
  onPlaybackProgress,
  onPlaybackStarted,
  retryCount,
  title,
  videoRef,
}: {
  adapter: MediaPlaybackAdapter;
  attachHlsJs: boolean;
  candidate: MediaPlaybackCandidate;
  fallbackDurationMs: number | null;
  onFailure(): void;
  onPlaybackEnded(snapshot: MediaPlaybackProgressSnapshot): void;
  onPlaybackPaused(snapshot: MediaPlaybackProgressSnapshot): void;
  onPlaybackProgress(snapshot: MediaPlaybackProgressSnapshot): void;
  onPlaybackStarted(): void;
  retryCount: number;
  title: string;
  videoRef: RefObject<HTMLVideoElement | null>;
}) {
  const onFailureRef = useRef(onFailure);

  useEffect(() => {
    onFailureRef.current = onFailure;
  }, [onFailure]);

  useEffect(() => {
    if (!attachHlsJs) {
      return;
    }

    const video = videoRef.current;
    const Hls = getHlsJsConstructor();
    if (!video || !Hls) {
      onFailureRef.current();
      return;
    }

    const hls = new Hls();
    const errorEvent = Hls.Events?.ERROR;
    if (errorEvent && typeof hls.on === "function") {
      hls.on(errorEvent, () => onFailureRef.current());
    }
    hls.loadSource(candidate.url);
    hls.attachMedia(video);

    return () => {
      hls.destroy();
    };
  }, [attachHlsJs, candidate.url, retryCount, videoRef]);

  return (
    <video
      aria-label={`${title} player`}
      className="mediaPlayer"
      controls
      data-playback-adapter={adapter.kind}
      onEnded={(event) =>
        onPlaybackEnded(mediaPlaybackProgressSnapshot(event.currentTarget, fallbackDurationMs))
      }
      onPause={(event) =>
        onPlaybackPaused(mediaPlaybackProgressSnapshot(event.currentTarget, fallbackDurationMs))
      }
      onError={onFailure}
      onPlay={onPlaybackStarted}
      onPlaying={onPlaybackStarted}
      onTimeUpdate={(event) =>
        onPlaybackProgress(mediaPlaybackProgressSnapshot(event.currentTarget, fallbackDurationMs))
      }
      key={`${candidate.key}:${retryCount}`}
      playsInline
      preload="metadata"
      ref={videoRef}
      src={attachHlsJs ? undefined : candidate.url}
    />
  );
}

function playbackTicketSignature(ticket: BrowserPlaybackTicketResponse) {
  return [
    ticket.item_id,
    ticket.mode,
    ticket.source_id,
    ticket.expires_at,
    ticket.urls
      .map(
        (url, index) =>
          `${index}:${url.kind}:${url.content_type}:${url.supports_range_requests}`,
      )
      .join("|"),
  ].join(":");
}

function playbackCandidates(ticket: BrowserPlaybackTicketResponse): MediaPlaybackCandidate[] {
  return ticket.urls.map((url, index) => ({
    contentType: url.content_type,
    index,
    key: `${ticket.source_id}:${index}:${url.kind}:${url.content_type}:${url.supports_range_requests}`,
    kind: url.kind,
    supportsRangeRequests: url.supports_range_requests,
    url: url.url,
  }));
}

function nextPlaybackCandidate(
  candidates: MediaPlaybackCandidate[],
  activeCandidateIndex: number,
) {
  return candidates.find((candidate) => candidate.index > activeCandidateIndex) ?? null;
}

function mediaPlaybackCandidateSelectionFor(
  selection: MediaPlaybackCandidateSelection,
  ticketSignature: string | null,
): MediaPlaybackCandidateSelection {
  if (selection.ticketSignature === ticketSignature) {
    return selection;
  }

  return {
    activeCandidateIndex: 0,
    failedCandidateKey: null,
    retryCount: 0,
    ticketSignature,
  };
}

function playbackAdapterFor(candidate: MediaPlaybackCandidate): MediaPlaybackAdapter {
  if (!isHlsCandidate(candidate)) {
    return { kind: "native-video", label: "browser stream" };
  }

  if (supportsNativeHlsPlayback()) {
    return { kind: "native-hls", label: "native HLS" };
  }

  if (supportsHlsJsPlayback()) {
    return { kind: "hls-js", label: "hls.js" };
  }

  return { kind: "unsupported-hls", label: "HLS unavailable" };
}

function isHlsCandidate(candidate: MediaPlaybackCandidate) {
  const contentType = candidate.contentType.toLowerCase();
  return (
    candidate.kind === "playlist" ||
    contentType.includes("mpegurl") ||
    contentType.includes("m3u8")
  );
}

function supportsNativeHlsPlayback() {
  if (typeof document === "undefined") {
    return false;
  }

  const video = document.createElement("video");
  if (typeof video.canPlayType !== "function") {
    return false;
  }

  return canPlay(video, "application/vnd.apple.mpegurl") || canPlay(video, "application/x-mpegURL");
}

function supportsHlsJsPlayback() {
  const Hls = getHlsJsConstructor();
  if (!Hls) {
    return false;
  }

  return typeof Hls.isSupported === "function" ? Hls.isSupported() : true;
}

function getHlsJsConstructor() {
  return (globalThis as typeof globalThis & { Hls?: HlsJsConstructor }).Hls;
}

function MediaPlaybackState({
  disabled,
  error,
  onMarkWatched,
  result,
  selectedSource,
}: {
  disabled: boolean;
  error: string | null;
  onMarkWatched(watched: boolean): void;
  result: MediaAsyncState<UserPlaybackStateResponse>;
  selectedSource: MediaSourceDto | undefined;
}) {
  if (result.loading) {
    return <div className="mediaSkeleton" />;
  }

  if (result.error) {
    return <div className="mediaError">{result.error}</div>;
  }

  const state = result.value?.state;
  const progress = state?.progress_percent ?? 0;

  return (
    <div className="mediaPlaybackState">
      <div>
        <strong>
          {state?.resume_position_ms
            ? `Continue from ${formatRuntimeMs(state.resume_position_ms)}`
            : "Start from beginning"}
        </strong>
        <span>{selectedSource?.file_name ?? "No source selected"}</span>
      </div>
      <progress value={progress} max={1} />
      <div className="mediaPlaybackActions">
        <Button
          disabled={disabled || state?.watched === true}
          onClick={() => onMarkWatched(true)}
          size="sm"
          type="button"
        >
          Mark watched
        </Button>
        <Button
          disabled={disabled || state?.watched === false}
          onClick={() => onMarkWatched(false)}
          size="sm"
          type="button"
          variant="outline"
        >
          Mark unwatched
        </Button>
      </div>
      {error ? <div className="mediaError">{error}</div> : null}
    </div>
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

function playbackDurationMs(
  item: ItemDetailResponse | null,
  decision: PlaybackDecisionResponse | null,
) {
  return (
    decision?.probe?.duration_ms ??
    (item?.item.metadata.runtime_minutes
      ? item.item.metadata.runtime_minutes * 60_000
      : null)
  );
}

function mediaPlaybackProgressSnapshot(
  video: HTMLVideoElement,
  fallbackDurationMs: number | null,
): MediaPlaybackProgressSnapshot {
  const durationMs = mediaSecondsToMs(video.duration) ?? fallbackDurationMs;
  const positionMs = mediaSecondsToMs(video.currentTime) ?? 0;
  return {
    durationMs,
    positionMs:
      durationMs && positionMs > durationMs ? durationMs : positionMs,
  };
}

function mediaSecondsToMs(value: number) {
  return Number.isFinite(value) && value > 0 ? Math.round(value * 1000) : null;
}

function browserPlaybackTicketRequest(
  decision: PlaybackDecisionResponse,
  capabilities: BrowserPlaybackCapabilityProfile,
): BrowserPlaybackTicketRequest {
  const mode = browserPlaybackMode(decision);
  return {
    capabilities: {
      ...capabilities,
      direct_play: mode === "direct",
      output_container: mode === "remux" ? "mp4" : undefined,
    },
    mode,
  };
}

function browserPlaybackMode(
  decision: PlaybackDecisionResponse,
): BrowserPlaybackTicketRequest["mode"] {
  if (decision.decision.mode === "direct_play") {
    return "direct";
  }
  if (decision.decision.mode === "remux") {
    return "remux";
  }
  return "hls";
}

function sourceSummary(source: MediaSourceDto) {
  return source.size_bytes ? "Local source" : "Source";
}

type BrowserPlaybackCapabilityProfile = NonNullable<
  BrowserPlaybackTicketRequest["capabilities"]
>;

const FALLBACK_BROWSER_PLAYBACK_CAPABILITIES: BrowserPlaybackCapabilityProfile = {
  audio_codec: ["aac", "opus", "mp3", "flac"],
  container: ["mp4", "webm", "mpegts"],
  direct_play: true,
  hls_segment_container: "fmp4",
  hls_variant_policy: "single_variant",
  output_container: "mp4",
  supports_hdr: false,
  supports_subtitles: true,
  video_codec: ["h264", "hevc", "vp9", "av1"],
};

function detectBrowserPlaybackCapabilities(): BrowserPlaybackCapabilityProfile {
  if (typeof document === "undefined") {
    return FALLBACK_BROWSER_PLAYBACK_CAPABILITIES;
  }

  const video = document.createElement("video");
  if (typeof video.canPlayType !== "function") {
    return FALLBACK_BROWSER_PLAYBACK_CAPABILITIES;
  }

  const supportsMp4H264 = canPlay(video, 'video/mp4; codecs="avc1.42E01E, mp4a.40.2"');
  const supportsMp4Hevc = canPlay(video, 'video/mp4; codecs="hvc1.1.6.L93.B0, mp4a.40.2"');
  const supportsWebmVp9 = canPlay(video, 'video/webm; codecs="vp9, opus"');
  const supportsWebmAv1 = canPlay(video, 'video/webm; codecs="av01.0.05M.08, opus"');
  const supportsNativeHls =
    canPlay(video, "application/vnd.apple.mpegurl") ||
    canPlay(video, "application/x-mpegURL");

  if (
    !supportsMp4H264 &&
    !supportsMp4Hevc &&
    !supportsWebmVp9 &&
    !supportsWebmAv1 &&
    !supportsNativeHls
  ) {
    return FALLBACK_BROWSER_PLAYBACK_CAPABILITIES;
  }

  const container = [
    supportsMp4H264 || supportsMp4Hevc ? "mp4" : null,
    supportsWebmVp9 || supportsWebmAv1 ? "webm" : null,
    supportsNativeHls ? "mpegts" : null,
  ].filter((value): value is string => Boolean(value));
  const videoCodec = [
    supportsMp4H264 ? "h264" : null,
    supportsMp4Hevc ? "hevc" : null,
    supportsWebmVp9 ? "vp9" : null,
    supportsWebmAv1 ? "av1" : null,
  ].filter((value): value is string => Boolean(value));
  const audioCodec = [
    supportsMp4H264 || supportsMp4Hevc ? "aac" : null,
    supportsWebmVp9 || supportsWebmAv1 ? "opus" : null,
    "mp3",
  ].filter((value): value is string => Boolean(value));

  return {
    audio_codec: audioCodec,
    container,
    direct_play: container.length > 0 && videoCodec.length > 0,
    hls_segment_container: supportsNativeHls ? "mpeg_ts" : "fmp4",
    hls_variant_policy: "single_variant",
    output_container: "mp4",
    supports_hdr: false,
    supports_subtitles: true,
    video_codec: videoCodec,
  };
}

function playbackCapabilitiesQuery(
  capabilities: BrowserPlaybackCapabilityProfile,
): PlaybackCapabilitiesQuery {
  return {
    audio_codec: capabilities.audio_codec,
    container: capabilities.container,
    direct_play: capabilities.direct_play,
    hls_segment_container: capabilities.hls_segment_container,
    hls_variant_policy: capabilities.hls_variant_policy,
    max_audio_channels: capabilities.max_audio_channels,
    max_height: capabilities.max_height,
    max_video_bitrate: capabilities.max_video_bitrate,
    max_width: capabilities.max_width,
    supports_hdr: capabilities.supports_hdr,
    supports_subtitles: capabilities.supports_subtitles,
    video_codec: capabilities.video_codec,
  };
}

function canPlay(video: HTMLVideoElement, mimeType: string) {
  const result = video.canPlayType(mimeType);
  return result === "maybe" || result === "probably";
}
