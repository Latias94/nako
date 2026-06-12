import type {
  ItemDetailResponse,
  MediaSourceDto,
  PlaybackDecisionResponse,
  UserPlaybackStateResponse,
} from "@nako/sdk";
import { useMemo, useState } from "react";

import { Button } from "../../components/ui/Button";
import {
  formatBytes,
  formatRuntimeMs,
  type MediaAsyncState,
  type MediaItemSearch,
  type MediaSearchChange,
  useMediaLoad,
} from "./MediaCore";
import {
  detectBrowserPlaybackCapabilities,
  playbackCapabilitiesQuery,
  playbackDurationMs,
  sourceSummary,
} from "./MediaPlaybackCore";
import type { MediaWebDataSource } from "./mediaDataSource";
import { useMediaSession } from "./MediaSession";

export type MediaItemPageProps = {
  itemId: string;
  onSearchChange: MediaSearchChange<MediaItemSearch>;
  search: MediaItemSearch;
};

export function useMediaItemPlayback({
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
    (source) =>
      source.getPlaybackDecision(
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
  const [playbackStateOverride, setPlaybackStateOverride] =
    useState<UserPlaybackStateResponse | null>(null);
  const [playbackMutationError, setPlaybackMutationError] = useState<string | null>(null);
  const [savingPlaybackState, setSavingPlaybackState] = useState(false);
  const fallbackDurationMs = playbackDurationMs(result.value, decision.value);

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
    setPlaybackStateOverride(null);
    setPlaybackMutationError(null);
  }

  return {
    browserCapabilities,
    dataSource,
    decision,
    fallbackDurationMs,
    mutationError: playbackMutationError,
    onMarkWatched: markWatched,
    onSourceChange: selectSource,
    playbackState: {
      ...playbackState,
      value: playbackStateOverride ?? playbackState.value,
    },
    result,
    savingPlaybackState,
    selectedSourceId,
    setPlaybackMutationError,
    setPlaybackStateOverride,
  };
}

export function MediaSourceVersions({
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

export function MediaPlaybackDecision({
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

export function MediaPlaybackState({
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

export type MediaPlaybackProgressSnapshot = {
  durationMs: number | null;
  positionMs: number;
};
export type { MediaAsyncState, MediaWebDataSource };
