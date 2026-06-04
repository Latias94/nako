# Playback HLS Session Lifecycle Extraction

## Goal

Move the remaining HLS playback-session playlist orchestration out of the broad
`PlaybackAppService` root and into the existing `app/playback/hls_flow.rs`
boundary. The user-facing behavior, public API shape, DTOs, schema, and
generated SDKs must remain unchanged.

This is a feature-backed fearless refactor: it keeps the already shipped HLS
startup/resource-admission behavior intact while making the server playback
root a thin entrypoint for HLS session lifecycle work, matching the recently
extracted Remux flow boundary.

## What I Already Know

* The previous Remux slice extracted source context construction, admission,
  background start, input staging/release, playback-session linkage, and output
  waiting into `app/playback/remux_flow.rs`.
* `app/playback/hls_flow.rs` already exists and owns source context
  construction, input staging, `HlsStart`/`HlsSupersede` resource admission,
  background HLS execution, and playlist readiness waiting.
* `PlaybackAppService::hls_source_with_policy` and
  `PlaybackAppService::hls_playlist_with_policy` already delegate to
  `hls_flow`.
* The remaining HLS lifecycle orchestration in `app/playback/mod.rs` is
  concentrated around `hls_playlist_playback` and
  `hls_playlist_for_playback_session`: effective policy lookup,
  playback-session creation/validation, transcode linkage, superseded playback
  session cancellation, and playback playlist artifact reading.
* `.trellis/spec/nako-server/backend/directory-structure.md` already defines a
  `Playback HLS Lifecycle Orchestration` scenario and expects
  `app/playback/hls_flow.rs` to own HLS lifecycle behavior.
* `docs/architecture/PLAYBACK.md` says ordinary HLS startup uses bounded
  `HlsStart`, replacement flows use `HlsSupersede`, and durable queueing /
  remote workers / LL-HLS remain follow-ons.

## Assumptions

* The task should be behavior-preserving and should not introduce a new HLS
  runtime capability.
* The task should not split `hls.rs`; that module remains the reserved HLS
  runner and transcode session persistence boundary around FFmpeg.
* The task should not move Direct Play, Renderer transport planning, Remux,
  segment routing, or artifact authority unless a tiny helper is needed for HLS
  flow reuse.

## Requirements

* Keep `PlaybackAppService` HLS playlist playback entrypoints thin:
  `hls_playlist_playback` and `hls_playlist_for_playback_session` should
  delegate to `hls_flow`.
* Move HLS playback-session orchestration into `hls_flow`, including:
  effective policy lookup for the source;
  `HlsSourceRequest` construction;
  playback-session creation or validation;
  transcode-session linkage;
  superseded HLS playback-session cancellation;
  linked transcode-session validation;
  playback playlist artifact reading.
* Preserve existing HLS resource-admission semantics:
  ordinary HLS startup uses bounded `HlsStart`;
  seek/replacement flows use bounded `HlsSupersede`;
  input staging happens only after configured capacity/admission checks.
* Preserve manifest-backed HLS playlist and segment authority through existing
  `hls_artifacts`, `playlist`, and `hls_artifact` helpers.
* Preserve trace request ID propagation through playlist playback startup.
* Preserve all existing public HTTP/API behavior.

## Acceptance Criteria

* [ ] `PlaybackAppService::hls_playlist_playback` delegates directly to
      `hls_flow::hls_playlist_playback`.
* [ ] `PlaybackAppService::hls_playlist_for_playback_session` delegates
      directly to `hls_flow::hls_playlist_for_playback_session`.
* [ ] `hls_flow.rs` owns HLS playback-session playlist orchestration without
      duplicating FFmpeg runner internals or playback compatibility rules.
* [ ] Existing HLS tests for source reuse, playlist readiness, seek supersede,
      bounded HLS start admission, trace context, and linked playback-session
      behavior still pass.
* [ ] Add focused coverage if the extraction exposes a previously untested
      HLS playback-session lifecycle path.
* [ ] No public API, DTO, schema, generated SDK, route shape, or error mapping
      change is introduced.

## Definition of Done

* Tests added or updated when risk justifies it.
* `cargo fmt --all -- --check` passes.
* `cargo check -p nako-server --tests` passes.
* Focused `cargo nextest run -p nako-server hls_playlist --no-fail-fast`
  passes.
* `git diff --check` passes.
* Trellis task context validates.
* Architecture/spec docs are updated only if implementation changes the
  durable boundary beyond the existing HLS lifecycle scenario.

## Technical Approach

Use the existing `hls_flow.rs` module as the extraction target. Move the two
session-facing playlist functions from `mod.rs` into `hls_flow.rs`, then keep
the `PlaybackAppService` methods as thin delegates. Reuse existing app-service
helpers for playback-session lookup, client capability extraction, policy
resolution, transcode lookup, linkage, cancellation, and artifact reading.

If helper visibility becomes awkward, prefer narrowly widening existing
`PlaybackAppService` helper visibility to `pub(super)` over duplicating logic.
Do not move repository trait definitions or broad shared helper functions out of
`mod.rs` during this task.

## Decision (ADR-lite)

**Context**: HLS source and playlist startup already have a focused
`hls_flow.rs` boundary, but HLS playback-session playlist orchestration still
lives in broad `mod.rs`.

**Decision**: Complete the boundary by moving HLS playback-session playlist
entrypoint orchestration into `hls_flow.rs`, while preserving `hls.rs` as the
runner boundary and leaving public contracts unchanged.

**Consequences**: Future HLS playlist/session lifecycle work has a smaller
module to modify. `mod.rs` remains the app-service root and shared helper
surface rather than the owner of HLS-specific orchestration. The immediate cost
is a small visibility adjustment for existing helpers.

## Out of Scope

* LL-HLS, CMAF, DASH, DRM, key delivery, remote transcode workers, durable HLS
  queueing, or per-artifact read/write pressure policy.
* New HLS output variants, FFmpeg argv changes, seek timestamp/keyframe policy,
  device-profile behavior, or compatibility planner changes.
* HTTP route rewrites, public DTO/schema changes, generated SDK updates, or
  browser/player UX changes.
* Splitting `hls.rs` runner internals or changing resource-admission policy
  semantics.

## Technical Notes

* Likely implementation files:
  * `crates/nako-server/src/app/playback/mod.rs`
  * `crates/nako-server/src/app/playback/hls_flow.rs`
  * `crates/nako-server/src/app/tests/playback.rs` if focused coverage is
    needed.
* Relevant specs:
  * `.trellis/spec/nako-server/backend/directory-structure.md`
  * `.trellis/spec/nako-server/backend/quality-guidelines.md`
* Relevant architecture map:
  * `docs/architecture/PLAYBACK.md`
* Current HLS flow boundary already owns:
  * `hls_source_with_policy`
  * `hls_playlist_with_policy`
  * HLS source context construction
  * `HlsStart` and `HlsSupersede` startup admission
  * background HLS start
  * playlist readiness waiting
