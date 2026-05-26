import { Search } from "lucide-react";
import { useEffect, useState, type FormEvent } from "react";

import { Button } from "../../components/ui/Button";
import type {
  MediaLoadResult,
  MediaConnection,
  MediaWebDataSource,
} from "./mediaDataSource";
import { useMediaSession } from "./MediaSession";
import type {
  ContinueWatchingResponse,
  ItemsResponse,
  LibraryListResponse,
  SearchResponse,
} from "@nako/sdk";

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

export function MediaLibrariesPage() {
  const { dataSource } = useMediaSession();
  const result = useMediaLoad(dataSource, (source) => source.listLibraries());

  if (!dataSource) {
    return <MediaConnectPage />;
  }

  return (
    <section className="mediaPage" aria-labelledby="media-libraries-title">
      <header className="mediaPageHeader">
        <h2 id="media-libraries-title">Media Libraries</h2>
        <span>{result.value?.page.returned ?? 0} accessible</span>
      </header>
      <MediaLibraryGrid result={result} />
    </section>
  );
}

export function MediaSearchPage() {
  const { dataSource } = useMediaSession();
  const [query, setQuery] = useState("");
  const [submittedQuery, setSubmittedQuery] = useState("");
  const result = useMediaLoad(
    dataSource,
    (source) => source.searchItems({ q: submittedQuery, limit: 20, offset: 0 }),
    [submittedQuery],
  );

  if (!dataSource) {
    return <MediaConnectPage />;
  }

  return (
    <section className="mediaPage" aria-labelledby="media-search-title">
      <header className="mediaPageHeader">
        <h2 id="media-search-title">Search</h2>
      </header>
      <form
        className="mediaSearch"
        onSubmit={(event) => {
          event.preventDefault();
          setSubmittedQuery(query.trim());
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
    </section>
  );
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
        <article className="mediaLibraryCard" key={library.id}>
          <span>{library.options.preset}</span>
          <strong>{library.name}</strong>
          <small>{library.options.metadata_profile.item_kinds.join(", ")}</small>
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
        <article className="mediaItemCard" key={item.id}>
          <span>{item.kind}</span>
          <strong>{item.metadata.title}</strong>
          <small>{item.metadata.runtime_minutes ? `${item.metadata.runtime_minutes} min` : "Runtime unknown"}</small>
        </article>
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
        <article className="mediaItemCard" key={hit.item.id}>
          <span>{Math.round(hit.score * 100)} match</span>
          <strong>{hit.item.metadata.title}</strong>
          <small>{hit.item.kind}</small>
        </article>
      ))}
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
