# Audio Compatibility Downmix Normalization

Status: Active
Last updated: 2026-05-30

## Why This Lane Exists

Nako can select audio streams and author HLS audio sidecars, but audio
compatibility is still mostly implicit. A 7.1 TrueHD or DTS-HD source on a
2.0 client is not solved by choosing any audio stream. The planner needs to
express output channel, codec, dynamic range, and loudness requirements before
transcode can plan reliable FFmpeg filters.

## Relevant Authority

- ADRs:
  - `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`
  - `docs/adr/0044-playback-capability-profile-planner.md`
  - `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
  - `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/architecture/PLAYBACK.md`
  - `docs/architecture/WORKSTREAM_LINKS.md`
  - `docs/architecture/LANES.md`
- Related workstreams:
  - `docs/workstreams/hls-alternate-audio-renditions/`
  - `docs/workstreams/hls-audio-sidecar-artifacts/`
  - `docs/workstreams/hls-selected-main-audio-cleanup/`
  - `docs/workstreams/playback-audio-language-default-policy/`

## Problem

The current planner vocabulary can explain selected audio and some compatibility
reasons, but it lacks a typed **Audio Output Requirement**. Transcode policy and
FFmpeg planning therefore cannot reliably distinguish direct-compatible audio,
codec conversion, channel downmix, dynamic range compression, loudness
normalization, or future dialogue clarity preferences.

## Target State

When this workstream closes:

- `nako-playback` models audio output requirements as playback-owned values;
- transcode policy receives audio compatibility requirements without rebuilding
  playback decisions;
- FFmpeg command planning can produce deterministic audio filter/output
  decisions for downmix and normalization;
- HLS runtime behavior remains compatible with selected main audio and sidecar
  media groups;
- diagnostics explain audio compatibility without leaking local paths or
  command-line details.

## In Scope

- playback-owned audio output requirement vocabulary;
- capability/profile matching for channels and basic audio compatibility;
- transcode profile/pipeline propagation;
- FFmpeg audio filter planning for downmix and normalization;
- focused playback, transcode, and HLS server tests.

## Out Of Scope

- HDR tone mapping;
- subtitle burn-in or image subtitle OCR;
- persisted per-user audio preferences and night-mode UI;
- web/mobile player controls;
- broad device profile database import;
- remastering, AI dialogue enhancement, or durable optimized audio assets.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Audio requirements should be expressed in `nako-playback` first. | High | Playback owns compatibility planning while transcode owns execution planning. | `nako-transcode` would start making playback policy decisions. |
| Initial downmix/normalization can be deterministic and profile-driven. | Medium | Current HLS policy already carries selected audio facts. | More client preference modeling may need a later lane. |
| HDR work must not run as implementation in parallel with this lane. | High | Both lanes touch playback/transcode planner and server playback seams. | Concurrent code changes would create merge and semantic conflicts. |

## Architecture Direction

Keep the direction consistent with prior playback seam work: `nako-playback`
owns client/source compatibility requirements; `nako-transcode` owns command and
pipeline execution planning; `nako-server` adapts runtime requests without
inventing new domain vocabulary.

The first executable slice is intentionally playback-only. Transcode and FFmpeg
tasks follow only after the requirement vocabulary is stable enough to review.

## Closeout Condition

This lane can close when:

- playback requirements, transcode propagation, and FFmpeg audio planning are
  all tested;
- HLS selected main audio and audio sidecar behavior remain compatible;
- diagnostics explain when downmix or normalization is selected;
- architecture docs and workstream evidence reflect shipped behavior;
- persisted preferences and UI controls are split or deferred.
