import { Link } from "@tanstack/react-router";
import type {
  ItemDetailResponse,
  PlaybackDecisionResponse,
  UserPlaybackStateResponse,
} from "@nako/sdk";

import { formatRuntimeMinutes, type MediaAsyncState } from "./MediaCore";
import { MediaConnectPage } from "./MediaConnectPage";
import {
  MediaPlaybackDecision,
  MediaPlaybackState,
  MediaSourceVersions,
  useMediaItemPlayback,
  type MediaItemPageProps,
} from "./MediaItemShared";

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
      onStartOver={playback.onStartOver}
      playbackState={playback.playbackState}
      result={playback.result.value}
      savingPlaybackState={playback.savingPlaybackState}
      selectedSourceId={playback.selectedSourceId}
    />
  );
}

function MediaItemDetail({
  decision,
  mutationError,
  onMarkWatched,
  onSourceChange,
  onStartOver,
  playbackState,
  result,
  savingPlaybackState,
  selectedSourceId,
}: {
  decision: MediaAsyncState<PlaybackDecisionResponse>;
  mutationError: string | null;
  onMarkWatched(watched: boolean): void;
  onSourceChange(sourceId: string): void;
  onStartOver(): void;
  playbackState: MediaAsyncState<UserPlaybackStateResponse>;
  result: ItemDetailResponse;
  savingPlaybackState: boolean;
  selectedSourceId: string | undefined;
}) {
  const metadata = result.item.metadata;
  const selectedSource =
    result.sources.find((source) => source.id === selectedSourceId) ?? result.sources[0];
  const resumeSourceId =
    playbackState.value?.state.resume_position_ms
      ? (playbackState.value.state.source_id ?? selectedSource?.id)
      : null;

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
          {resumeSourceId ? (
            <Link
              className="uiButton uiButtonOutline uiButtonSm"
              params={{ itemId: result.item.id }}
              search={{ source_id: resumeSourceId }}
              to="/media/watch/$itemId"
            >
              Resume
            </Link>
          ) : null}
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
          onStartOver={onStartOver}
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
