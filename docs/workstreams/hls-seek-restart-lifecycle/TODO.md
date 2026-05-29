# HLS Seek Restart Lifecycle - TODO

Status: Active
Last updated: 2026-05-29

Task IDs use the `HSRL` prefix.

## M0 - Lane Setup

- [x] HSRL-010 [owner=codex] [deps=none] [scope=docs/workstreams/hls-seek-restart-lifecycle,docs/architecture,docs/workstreams/README.md]
  Goal: Open the workstream, define the lifecycle boundary, and link the lane
  from architecture indexes.
  Validation: `python3 -m json.tool docs/workstreams/hls-seek-restart-lifecycle/WORKSTREAM.json`
  Evidence: `README.md`, `DESIGN.md`, `TODO.md`, `WORKSTREAM.json`
  Handoff: Continue with HSRL-020.

## M1 - Request Identity And Generation Model

- [x] HSRL-020 [owner=codex] [deps=HSRL-010] [scope=crates/nako-transcode/src/artifact.rs,crates/nako-transcode/src/lib.rs,crates/nako-server/src/app/playback,crates/nako-server/src/app/tests/playback.rs]
  Goal: Add an internal HLS playback generation/start-position identity
  component that preserves default request keys and isolates non-zero seek
  generations.
  Validation: `cargo nextest run -p nako-transcode hls_request_variant --no-fail-fast`; `cargo nextest run -p nako-server hls_source_request_identity --no-fail-fast`
  Evidence: `HlsPlaybackGeneration` is part of `HlsRequestVariantPlan`; default
  generation stays out of identity keys, and non-zero start positions isolate
  request identity/staging layout. Targeted validation passed.
  Handoff: DONE. Continue with HSRL-030.

## M2 - Restart Admission Policy

- [x] HSRL-030 [owner=codex] [deps=HSRL-020] [scope=crates/nako-server/src/app/playback/hls.rs,crates/nako-server/src/app/playback/mod.rs,crates/nako-server/src/app/tests/playback.rs]
  Goal: Model same-generation reuse vs superseding-generation restart and make
  cancellation/admission behavior explicit.
  Validation: `cargo nextest run -p nako-server hls_source --no-fail-fast`
  Evidence: same request key active duplicates still conflict; finished same
  request key sessions still reuse; non-zero seek generation creates a new
  request key and marks prior active same-source HLS sessions as cancellation
  requested.
  Handoff: DONE. Continue with HSRL-040.

## M3 - FFmpeg Seek Command Planning

- [ ] HSRL-040 [owner=codex] [deps=HSRL-030] [scope=crates/nako-transcode,crates/nako-server/src/app/playback]
  Goal: Pass generation start position into HLS command planning with explicit
  seek flags and timestamp/keyframe behavior tests.
  Validation: `cargo nextest run -p nako-transcode hls --no-fail-fast`; `cargo nextest run -p nako-server hls --no-fail-fast`
  Evidence: FFmpeg argv tests and server playback tests.
  Handoff: Continue with HSRL-050.

## M4 - Public Playback Integration And Closeout

- [ ] HSRL-050 [owner=codex] [deps=HSRL-040] [scope=crates/nako-api,crates/nako-server,docs/workstreams/hls-seek-restart-lifecycle]
  Goal: Decide and implement the public/internal seek request surface, record
  closeout evidence, and split remaining client-player work.
  Validation: focused API/server gates plus closeout checks from
  `EVIDENCE_AND_GATES.md`
  Evidence: DTO/server tests or explicit no-wire-change decision.
  Handoff: DONE or split follow-ons.
