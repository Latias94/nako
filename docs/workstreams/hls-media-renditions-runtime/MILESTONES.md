# HLS Media Renditions Runtime Milestones

Status: Closed
Last updated: 2026-05-29

## Milestone 1 - Workstream Opened

Status: Done

- Created durable docs for the HLS media renditions runtime lane.
- Split this first stage from LL-HLS, DRM, full alternate audio UX, subtitle
  OCR, and second-engine adapter work.

## Milestone 2 - Typed Rendition Plan

Status: Done

- HLS runtime has a typed media rendition plan for selected subtitles and future
  alternate audio.
- Request identity can carry rendition decisions that affect artifact shape.
- `HlsRequestVariantPlan` now bundles source-aware adaptive ladder decisions
  with selected subtitle media rendition decisions.

## Milestone 3 - First Executable Slice

Status: Done

- Selected subtitle WebVTT execution is implemented, or the typed foundation is
  landed and extraction is split with evidence.
- Selected subtitles now emit WebVTT sidecar playlist and segment artifacts via
  the FFmpeg HLS command planner.

## Milestone 4 - Server Runtime Verified And Closed

Status: Done

- Server HLS staging, artifact serving, playlist rewrite, reuse, and redaction
  gates pass.
- Workstream evidence is recorded and status is closed.
- Focused transcode, playback, server HLS, server playback, format, and diff
  gates are recorded in `EVIDENCE_AND_GATES.md`.
