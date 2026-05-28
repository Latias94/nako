# Playback Runtime Boundary Deepening - Evidence And Gates

Status: Completed
Last updated: 2026-05-28

## Smallest Current Repro

```bash
cargo nextest run -p nako-server hls --no-fail-fast
```

This gate proves HLS playlist/segment behavior while the first slice moves
artifact serving out of the broad playback app module.

## Gate Set

### Targeted Iteration Gates

```bash
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server admin_v1_playback --no-fail-fast
```

### Package Gate

```bash
cargo nextest run -p nako-server playback --no-fail-fast
```

### Closeout Gate

```bash
cargo fmt --all -- --check
git diff --check
python3 -m json.tool docs/workstreams/playback-runtime-boundary-deepening/WORKSTREAM.json
```

Use focused server gates for closeout unless the refactor crosses public API or
database contracts.

### Review Gate

Run `review-workstream` before accepting task or lane completion. Record
blocking findings, missing gates, and residual risks here or in `HANDOFF.md`.

## Evidence Anchors

- `docs/workstreams/playback-runtime-boundary-deepening/DESIGN.md`
- `docs/workstreams/playback-runtime-boundary-deepening/TODO.md`
- `docs/workstreams/playback-runtime-boundary-deepening/MILESTONES.md`
- `docs/workstreams/playback-runtime-boundary-deepening/HANDOFF.md`
- `crates/nako-server/src/app/playback`
- `crates/nako-server/src/http/tests/playback.rs`

## Evidence Log

- 2026-05-28 PRBD-010: Opened the workstream and froze scope around
  behavior-preserving playback runtime boundary deepening. Validation target:
  `python3 -m json.tool docs/workstreams/playback-runtime-boundary-deepening/WORKSTREAM.json`.
- 2026-05-28 PRBD-020: Extracted HLS artifact serving into
  `crates/nako-server/src/app/playback/hls_artifact.rs`. The new boundary owns
  playback-session playlist rewrite, playable session state checks, progressive
  segment readiness, throttled wait, stale sibling `.ts` cleanup, and segment
  response planning. Validation passed with `cargo nextest run -p nako-server
  hls --no-fail-fast` and a direct rerun of
  `hls_segment_cleanup_removes_stale_siblings_and_keeps_requested`.
- 2026-05-28 PRBD-030: Extracted server-side playback support evidence context
  and runtime diagnostics collection into
  `crates/nako-server/src/app/playback/support.rs`. Admin DTO mapping remains
  in the HTTP/Admin layer and no wire contract changed. Validation passed with
  `cargo nextest run -p nako-server admin_v1_playback --no-fail-fast`.
- 2026-05-28 PRBD-040: Audited store ports and test locality after the HLS and
  support extractions. `hls_artifact.rs` has no store dependency and owns the
  HLS artifact unit tests. `support.rs` uses only source/session lookup, but a
  private two-method trait would add pass-through churn because the module is
  only called by `PlaybackAppService`. HLS/remux execution store narrowing
  remains a possible future lane. Validation passed with `cargo nextest run -p
  nako-server playback --no-fail-fast` (87 tests).
- 2026-05-28 PRBD-050: Closeout review found no blocking workstream compliance
  or code-quality findings. Final focused playback gate passed with nextest
  run `af7c9b69-0399-47fe-bd4a-d5ff31e150d2`:
  `cargo nextest run -p nako-server playback --no-fail-fast` (87 passed, 296
  skipped). Final non-test checks passed; see `CLOSEOUT.md`.

## Notes

- Do not add adaptive HLS, fMP4, rsmpeg, subtitle/HDR, or remote worker
  behavior in this lane.
- A narrower trait is only a win when it deletes coupling. Pass-through traits
  are a follow-on smell, not progress.
- Fresh verification is required before marking a task, Codex goal, or lane
  complete.
