import type {
  BrowserPlaybackTicketRequest,
  MediaSourceDto,
  PlaybackCapabilitiesQuery,
  PlaybackDecisionResponse,
} from "@nako/sdk";

export const MEDIA_PROGRESS_WRITE_INTERVAL_MS = 30_000;

export function playbackDurationMs(
  item: { item: { metadata: { runtime_minutes: number | null } } } | null,
  decision: PlaybackDecisionResponse | null,
) {
  return (
    decision?.probe?.duration_ms ??
    (item?.item.metadata.runtime_minutes ? item.item.metadata.runtime_minutes * 60_000 : null)
  );
}

export function browserPlaybackTicketRequest(
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

export function browserPlaybackMode(
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

export function sourceSummary(source: MediaSourceDto) {
  return source.size_bytes ? "Local source" : "Source";
}

export type BrowserPlaybackCapabilityProfile = NonNullable<
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

export function detectBrowserPlaybackCapabilities(): BrowserPlaybackCapabilityProfile {
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

export function playbackCapabilitiesQuery(
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

export function canPlay(video: HTMLVideoElement, mimeType: string) {
  const result = video.canPlayType(mimeType);
  return result === "maybe" || result === "probably";
}
