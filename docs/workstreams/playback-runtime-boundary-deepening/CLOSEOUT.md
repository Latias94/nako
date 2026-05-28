# Playback Runtime Boundary Deepening - Closeout

Status: Completed
Last updated: 2026-05-28

## Final Status

Closed on 2026-05-28. PRBD-010 through PRBD-050 are complete.

The lane preserved playback behavior while moving cohesive runtime read-model
and artifact-serving responsibilities out of the broad playback app module.

## Shipped Boundary Changes

- Added `crates/nako-server/src/app/playback/hls_artifact.rs` for HLS playback
  playlist rewrite, playable state checks, segment readiness, throttled waits,
  stale segment cleanup, and segment response planning.
- Added `crates/nako-server/src/app/playback/support.rs` for server-side
  playback support evidence context and runtime diagnostics collection.
- Reduced `crates/nako-server/src/app/playback/mod.rs` from roughly 2451 lines
  to roughly 2082 lines without route or DTO changes.
- Kept `PlaybackRuntimeStore` unsplit in this lane. The new HLS artifact module
  does not need store access, and a private two-method support trait would be
  pass-through churn.

## Review Result

No blocking workstream compliance or code-quality findings remain.

Important review notes:

- Public/Admin API behavior and redaction semantics are unchanged.
- HLS artifact behavior remains covered through route tests and local
  `hls_artifact` unit tests.
- Runtime-store narrowing should only be revisited if HLS/remux execution
  orchestration grows enough to justify a real execution-store port.

## Verification

Fresh gates:

- `python3 -m json.tool docs/workstreams/playback-runtime-boundary-deepening/WORKSTREAM.json`
- `cargo nextest run -p nako-server hls --no-fail-fast` passed: 19 tests.
- `cargo nextest run -p nako-server admin_v1_playback --no-fail-fast` passed:
  9 tests.
- `cargo nextest run -p nako-server playback --no-fail-fast` passed: 87 tests,
  296 skipped; nextest run `af7c9b69-0399-47fe-bd4a-d5ff31e150d2`.
- `cargo fmt --all -- --check`
- `git diff --check`

## Follow-Ons

- HLS/remux execution store-port narrowing, only if execution orchestration
  grows further.
- Adaptive HLS/fMP4 output maturity.
- Subtitle/audio/HDR transcode maturity.
- rsmpeg adapter feasibility.

## Residual Risk

- `PlaybackAppService` remains a broad composition entry point. This is
  intentional for now; deeper store/execution port splits should be justified
  by future feature pressure rather than done mechanically.
