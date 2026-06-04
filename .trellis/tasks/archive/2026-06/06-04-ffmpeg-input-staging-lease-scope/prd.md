# FFmpeg Input Staging Lease Scope

## Goal

Deepen the server Playback Runtime input-staging module so HLS and Remux flows
no longer manually remember to release staged FFmpeg input leases on every
success/error path.

## Intent

Remote Media Sources are staged locally before FFmpeg receives an input path.
`FfmpegInputService` already hides local-path vs staged-input selection, but the
callers still own the lease release invariant. This refactor should move that
invariant behind a scoped async interface, improving locality for remote
staging cleanup and reducing future leak risk.

## Scope

Primary files:

* `crates/nako-server/src/app/playback/input.rs`
* `crates/nako-server/src/app/playback/hls_flow.rs`
* `crates/nako-server/src/app/playback/remux_flow.rs`
* focused playback app tests under `crates/nako-server/src/app/tests/`

Reference findings:

* `.trellis/tasks/archive/2026-06/06-04-architecture-boundary-refactor-review/report.md`
* `.trellis/tasks/archive/2026-06/06-04-architecture-boundary-refactor-review/research/playback-runtime-boundaries.md`

## Requirements

* Add a scoped async interface to `FfmpegInputService` so callers provide an
  async operation that receives the staged/local input path.
* Keep staged input lease acquisition and release local to
  `FfmpegInputService`.
* Preserve current behavior:
  * local path input does not acquire a staging lease;
  * remote/staged input acquires and releases the staging manifest lease;
  * on runner success, release errors remain returned to the caller;
  * on runner error, release errors are logged and the original runner error is
    returned.
* Replace manual HLS and Remux release blocks with the scoped interface.
* Do not change public HTTP/API DTOs, transcode command planning, playback
  planning rules, schema, or resource admission semantics.

## Refactor Brief

### Deletion Plan

* Delete duplicated `match result { Ok(output) => release; Err(err) => log
  release error; Err(err) }` blocks from HLS and Remux flow code.
* Avoid adding a second lease guard or fire-and-forget release path.

### Boundary Plan

* `FfmpegInputService` owns:
  * local path hint detection;
  * VFS staging through `ManifestRecordingStorageBackend`;
  * `StagingLease` acquisition and release;
  * success/error release policy.
* HLS and Remux flows receive only a path inside the scoped operation and do not
  know whether a lease exists.
* No dependency changes and no new crate seam.

### Testing Plan

* Add or update focused app tests proving:
  * local input path does not require lease release side effects;
  * staged input release happens after successful HLS/Remux-style operation;
  * staged input release happens after an operation error;
  * when release fails after an operation error, the original operation error is
    preserved and the release failure is only logged.
* Run:
  * `cargo check -p nako-server --tests`
  * `cargo nextest run -p nako-server ffmpeg_input --no-fail-fast`
  * relevant HLS/Remux focused filters if touched behavior requires them
  * `cargo fmt --all -- --check`
  * `git diff --check`

### Risk Plan

* Async closures in Rust can create lifetime friction. Prefer a boxed future or
  helper trait only if a simple generic scoped API is not ergonomic.
* Do not rely on `Drop` for async release.
* Do not widen `FfmpegSourceInput` visibility or make callers inspect lease
  state.
* Avoid changing HLS/Remux resource admission order.

## Acceptance Criteria

* [x] HLS flow uses the scoped input interface and no longer manually releases
      `FfmpegSourceInput`.
* [x] Remux flow uses the scoped input interface and no longer manually releases
      `FfmpegSourceInput`.
* [x] `FfmpegInputService` tests cover success and error release behavior.
* [x] Existing HLS/Remux behavior remains compatible.
* [x] Focused server checks pass.

## Verification Evidence

* `cargo check -p nako-server --tests`
* `cargo nextest run -p nako-server ffmpeg_input --no-fail-fast`
* `cargo nextest run -p nako-server remote_staged_input --no-fail-fast`
* `cargo nextest run -p nako-server remux --no-fail-fast`
* `cargo nextest run -p nako-server hls_source --no-fail-fast`
* `cargo nextest run -p nako-server hls_playlist --no-fail-fast`
* `cargo fmt --all -- --check`
* `git diff --check`
* `python ./.trellis/scripts/task.py validate .trellis/tasks/06-04-ffmpeg-input-staging-lease-scope`

## Definition of Done

* Code is formatted.
* Focused `nako-server` check/tests pass.
* Trellis context is validated.
* No public API/schema/docs changes unless implementation discovers a real
  contract issue.

## Out of Scope

* Playback Transcode Runtime session unification.
* Playback Source Context builder work.
* HLS Artifact Authority.
* Direct Play media transport response refactor.
* Any DB migration, public route, Admin API, or generated contract change.
