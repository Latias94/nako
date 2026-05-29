# Playback Planner Transcode Seam Deepening

Status: Completed
Last updated: 2026-05-29

This fearless refactor lane tightens the boundary between playback planning and
transcode execution. `nako-playback` should decide what kind of rendition is
needed and preserve the source/client/policy facts behind that decision.
`nako-transcode` should own `TranscodeProfile` construction, validation,
identity material, and execution-facing profile vocabulary.

## Completed Result

- `nako-transcode` owns playback remux/HLS profile request builders.
- `PlaybackTargetProfile` no longer constructs `TranscodeProfile` values.
- Server playback orchestration composes planner output with runtime execution
  policy through transcode-owned builders.
- Remux/HLS request identity and playback runtime behavior remain covered by
  focused nextest gates.

## Why Now

Recent HLS, adaptive ladder, media rendition, and audio sidecar work made the
runtime deeper, but `PlaybackTargetProfile` still constructs execution-ready
`TranscodeProfile` values. That makes the planner aware of transcode execution
shapes just before future work adds HLS seek/restart, HDR tone mapping,
downmix/normalization, and resource scheduling.

## Non-Goals

- No HLS seek/restart behavior was added.
- No HDR tone mapping, audio downmix, or subtitle burn-in behavior was added.
- No wire DTO, schema migration, or public API contract changed.
- No new media engine or FFmpeg command behavior was added.

## Architecture References

- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`
- `docs/adr/0044-playback-capability-profile-planner.md`
- `docs/adr/0049-source-aware-transcode-runtime.md`
