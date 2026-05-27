# Source-Aware Transcode Runtime

Status: Completed
Last updated: 2026-05-28

This workstream deepens Nako's playback transcode stack from a fixed HLS
H.264/AAC path into a source-aware runtime. The lane uses Jellyfin, FFmpeg, and
rsmpeg as reference pressure only; Nako keeps its own domain records, planning
boundaries, and Rust module shape.

## Goals

- Make media probe facts rich enough to explain decoder, filter, HDR,
  subtitle, audio, and HLS output decisions.
- Turn playback incompatibility into a structured `TranscodeRequirement`
  before choosing FFmpeg or hardware details.
- Make `TranscodePipelinePlanner` source-aware so decode, filter, encode, and
  fallback decisions depend on input codec facts and output constraints.
- Split FFmpeg HLS command construction into device, input, filter, encoder,
  and muxer concerns without adopting Jellyfin's monolithic helper shape.
- Add the runtime hooks needed for progressive HLS, progress telemetry,
  cancellation, throttling, and segment cleanup.
- Keep Public Client responses redaction-safe while Admin diagnostics can
  explain concrete capability gaps.

## Non-Goals

- Copy Jellyfin source, schemas, comments, tests, assets, or generated output.
- Replace FFmpeg CLI with rsmpeg in this lane.
- Implement every hardware backend and codec combination before the first
  source-aware planner proof.
- Build a frontend player UI.
- Add DLNA, SyncPlay, live TV, offline sync, or remote transcode workers.
- Change plugin/addon architecture.

## Reference Pressure

- `repo-ref/jellyfin/MediaBrowser.MediaEncoding/Encoder/EncoderValidator.cs`
  for decoder, encoder, filter, hwaccel, and bitstream-filter validation
  pressure.
- `repo-ref/jellyfin/MediaBrowser.Controller/MediaEncoding/EncodingHelper.cs`
  for source-aware hardware decode, device initialization, and filter graph
  pressure.
- `repo-ref/jellyfin/MediaBrowser.MediaEncoding/Transcoding/TranscodeManager.cs`
  for transcode job lifecycle, progress, cancellation, throttling, and output
  cleanup pressure.
- `repo-ref/jellyfin/MediaBrowser.Model/Configuration/EncodingOptions.cs` for
  operator settings pressure around devices, tone mapping, subtitle extraction,
  throttling, and codec-specific hardware decode gates.
- `repo-ref/ffmpeg/doc/ffmpeg.texi`, `repo-ref/ffmpeg/doc/filters.texi`, and
  `repo-ref/ffmpeg/doc/muxers.texi` for authoritative FFmpeg CLI, hardware
  device, filter, and HLS muxer behavior.
- `repo-ref/rsmpeg` for future typed FFmpeg API ergonomics; it is not the first
  execution adapter target.

## Authoritative Docs

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`
