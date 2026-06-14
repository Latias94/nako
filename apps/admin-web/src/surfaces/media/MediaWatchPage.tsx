import type {
  BrowserPlaybackTicketResponse,
  ItemDetailResponse,
  PlaybackDecisionResponse,
  UserPlaybackStateResponse,
} from "@nako/sdk";
import { ArrowRight, RefreshCw } from "lucide-react";
import { useCallback, useEffect, useRef, useState, type RefObject } from "react";

import { Button } from "../../components/ui/Button";
import {
  formatRuntimeMs,
  type MediaAsyncState,
  useMediaLoad,
} from "./MediaCore";
import { MediaConnectPage } from "./MediaConnectPage";
import {
  MediaPlaybackDecision,
  MediaPlaybackState,
  MediaSourceVersions,
  useMediaItemPlayback,
  type MediaItemPageProps,
  type MediaPlaybackProgressSnapshot,
  type MediaWebDataSource,
} from "./MediaItemShared";
import {
  MEDIA_PROGRESS_WRITE_INTERVAL_MS,
  browserPlaybackTicketRequest,
  canPlay,
  playbackDurationMs,
} from "./MediaPlaybackCore";

export function MediaWatchPage(props: MediaItemPageProps) {
  const playback = useMediaItemPlayback(props);
  const [browserTicketRetryKey, setBrowserTicketRetryKey] = useState(0);
  const browserTicket = useMediaLoad(
    playback.selectedSourceId && playback.decision.value ? playback.dataSource : null,
    (source) =>
      source.createBrowserPlaybackTicket(
        playback.selectedSourceId!,
        browserPlaybackTicketRequest(playback.decision.value!, playback.browserCapabilities),
      ),
    [
      playback.selectedSourceId,
      playback.decision.value?.decision.mode,
      playback.decision.value?.decision.transcode_plan?.output_container,
      playback.browserCapabilities,
      browserTicketRetryKey,
    ],
  );
  const playbackProgress = useMediaPlaybackProgress({
    dataSource: playback.dataSource,
    fallbackDurationMs: playback.fallbackDurationMs,
    itemId: props.itemId,
    selectedSourceId: playback.selectedSourceId,
    setPlaybackMutationError: playback.setPlaybackMutationError,
    setPlaybackStateOverride: playback.setPlaybackStateOverride,
  });
  const selectSource = (sourceId: string) => {
    playback.onSourceChange(sourceId);
    setBrowserTicketRetryKey(0);
  };

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
      browserTicket={browserTicket}
      decision={playback.decision}
      mutationError={playback.mutationError}
      onMarkWatched={playback.onMarkWatched}
      onBrowserTicketRetry={() => setBrowserTicketRetryKey((current) => current + 1)}
      onPlaybackEnded={playbackProgress.onEnded}
      onPlaybackPaused={playbackProgress.onPaused}
      onPlaybackProgress={playbackProgress.onProgress}
      onPlaybackStarted={playbackProgress.onStarted}
      onSourceChange={selectSource}
      playbackState={playback.playbackState}
      result={playback.result.value}
      savingPlaybackState={playback.savingPlaybackState}
      selectedSourceId={playback.selectedSourceId}
    />
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
          <span>{metadata.runtime_minutes ? `${metadata.runtime_minutes} min` : "Runtime unknown"}</span>
        </div>
      </header>
      <section className="mediaPanel mediaPlayerShell" aria-labelledby="media-player-title">
        <div className="mediaPanelHeader">
          <h3 id="media-player-title">Player</h3>
          <span>{browserTicket.value?.mode ?? decision.value?.decision.mode ?? "pending"}</span>
        </div>
        <MediaResumeSummary result={playbackState} selectedSourceId={selectedSource?.id} />
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

function MediaResumeSummary({
  result,
  selectedSourceId,
}: {
  result: MediaAsyncState<UserPlaybackStateResponse>;
  selectedSourceId: string | undefined;
}) {
  if (result.loading) {
    return null;
  }

  const state = result.value?.state;
  if (!state) {
    return null;
  }

  const progressPercent = Math.round((state.progress_percent ?? 0) * 100);
  const isSelectedSource = state.source_id === selectedSourceId;

  return (
    <div className="mediaResumeSummary">
      <div>
        <strong>
          {state.resume_position_ms
            ? `Resume from ${formatRuntimeMs(state.resume_position_ms)}`
            : "Start from beginning"}
        </strong>
        <span>
          {state.watched ? "Watched" : `${progressPercent}% complete`}
          {" - "}
          {isSelectedSource ? "current source" : "different saved source"}
        </span>
      </div>
      <progress value={state.progress_percent ?? 0} max={1} />
    </div>
  );
}

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

function mediaPlaybackProgressSnapshot(
  video: HTMLVideoElement,
  fallbackDurationMs: number | null,
): MediaPlaybackProgressSnapshot {
  const durationMs = mediaSecondsToMs(video.duration) ?? fallbackDurationMs;
  const positionMs = mediaSecondsToMs(video.currentTime) ?? 0;
  return {
    durationMs,
    positionMs: durationMs && positionMs > durationMs ? durationMs : positionMs,
  };
}

function mediaSecondsToMs(value: number) {
  return Number.isFinite(value) && value > 0 ? Math.round(value * 1000) : null;
}
