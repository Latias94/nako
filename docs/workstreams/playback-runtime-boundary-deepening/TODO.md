# Playback Runtime Boundary Deepening - TODO

Status: Completed
Last updated: 2026-05-28

Task IDs use the `PRBD` prefix.

## M0 - Scope And Evidence Freeze

- [x] PRBD-010 [owner=codex] [deps=none] [scope=docs/workstreams/playback-runtime-boundary-deepening,docs/workstreams/README.md]
  Goal: Open the fearless refactor lane, freeze scope, task order, non-goals,
  and validation gates.
  Validation: `python3 -m json.tool docs/workstreams/playback-runtime-boundary-deepening/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `TODO.md`, `WORKSTREAM.json`
  Handoff: Continue with PRBD-020.

## M1 - HLS Artifact Serving Boundary

- [x] PRBD-020 [owner=codex] [deps=PRBD-010] [scope=crates/nako-server/src/app/playback]
  Goal: Move HLS playback-session playlist rewrite, playable state checks,
  segment readiness, throttled wait, segment cleanup, and segment response
  planning into a focused HLS artifact module/service while preserving route
  behavior.
  Validation: `cargo nextest run -p nako-server hls --no-fail-fast`
  Review: pending `review-workstream` for route behavior and lifecycle
  locality.
  Evidence: `hls_artifact.rs` owns playback playlist rewrite, playable state
  checks, segment readiness, throttled wait, stale segment cleanup, and segment
  response planning. `cargo nextest run -p nako-server hls --no-fail-fast`
  passed; focused cleanup test also passed when rerun directly.
  Handoff: DONE. Continue with PRBD-030.

## M2 - Support Evidence And Diagnostics Boundary

- [x] PRBD-030 [owner=codex] [deps=PRBD-020] [scope=crates/nako-server/src/app/playback,crates/nako-server/src/http/tests]
  Goal: Extract Admin support evidence/runtime diagnostics collection into a
  read-model boundary or explicitly document why it should remain on
  `PlaybackAppService`.
  Validation: `cargo nextest run -p nako-server admin_v1_playback --no-fail-fast`
  Review: pending `review-workstream` for redaction and Admin/Public boundary
  stability.
  Evidence: `support.rs` owns server-side support evidence context and playback
  runtime diagnostics collection. `cargo nextest run -p nako-server
  admin_v1_playback --no-fail-fast` passed with 9 tests.
  Handoff: DONE. Admin DTOs were unchanged.

## M3 - Store Port And Test Locality Audit

- [x] PRBD-040 [owner=codex] [deps=PRBD-030] [scope=crates/nako-server/src/app/playback]
  Goal: Audit whether HLS artifact/support/remux sub-boundaries can use
  narrower store ports or better local tests; split only seams that remove real
  coupling.
  Validation: `cargo nextest run -p nako-server playback --no-fail-fast`
  Review: pending `review-workstream` for trait churn and module locality.
  Evidence: `hls_artifact.rs` has no store dependency and owns local tests;
  `support.rs` currently needs only lookup calls but is private and called only
  by `PlaybackAppService`, so a two-method store trait would be pass-through
  churn. HLS/remux execution store narrowing is split as a future runtime-store
  lane if needed. `cargo nextest run -p nako-server playback --no-fail-fast`
  passed with 87 tests.
  Handoff: DONE. No store trait split in this lane.

## M4 - Closeout

- [x] PRBD-050 [owner=codex] [deps=PRBD-040] [scope=docs/workstreams/playback-runtime-boundary-deepening]
  Goal: Verify the lane, record evidence, close or split remaining work, and
  update workstream status.
  Validation: `cargo fmt --all -- --check`, `git diff --check`, focused gates
  from `EVIDENCE_AND_GATES.md`
  Review: completed with no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`,
  `CLOSEOUT.md`
  Handoff: DONE. Follow-ons are HLS/remux execution store-port narrowing,
  adaptive HLS/fMP4, subtitle/audio/HDR maturity, and rsmpeg adapter
  feasibility.
