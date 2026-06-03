# brainstorm: hls ffmpeg builder parity

## Goal

Deepen the `nako-transcode` HLS FFmpeg command-builder seam so future
Jellyfin-grade HLS work around subtitles, HDR, audio renditions, and adaptive
ladders remains typed, exact-testable, and local to the FFmpeg adapter.

## What I already know

* Prior transcode parity work concluded that a wholesale rewrite is not
  justified.
* The next open candidate is `crates/nako-transcode/src/ffmpeg/hls.rs`.
* HLS command planning is already split into `encoders`, `filters`, `input`,
  `muxer`, `seek`, and `sidecars` submodules.
* The refactor should deepen the existing builder shape, not move playback
  decision logic or artifact authority into the FFmpeg layer.

## Assumptions

* "Continue" means proceed with the third bounded architecture slice from the
  archived transcode parity task.
* The MVP should preserve behavior and public API shape.
* Exact argv tests are the quality bar for any FFmpeg command-planning change.

## Requirements

* Inspect current HLS FFmpeg command planning and identify one bounded seam
  that reduces future caller-side branching or command-fragment drift.
* Preserve typed request input through `HlsRequest`, `HlsArtifactManifest`, and
  `TranscodeExecutionPolicy`.
* Do not concatenate ad hoc command strings outside existing `FfmpegArg`
  builders.
* Do not copy or translate Jellyfin implementation code.
* Keep the refactor inside `nako-transcode` unless a compile-time contract
  requires a narrow caller adjustment.
* Add or update focused exact-argv tests for the modified command-planning
  behavior.

## Acceptance Criteria

* [x] A bounded HLS FFmpeg builder seam is deepened without changing behavior.
* [x] Public API shape remains stable unless a deliberately justified internal
  signature change is required.
* [x] HLS subtitles, audio sidecars, HDR/filter, or adaptive ladder command
  paths remain typed and exact-testable.
* [x] Focused `nako-transcode` nextest filters pass.
* [x] `cargo check -p nako-transcode --tests` passes.
* [x] Relevant Trellis spec/research notes are updated if a reusable convention
  is discovered.

## Definition of Done

* Tests added or updated where the command shape changes.
* `cargo fmt --all -- --check` passes.
* Focused `cargo nextest run -p nako-transcode <filter> --no-fail-fast` passes.
* `cargo check -p nako-transcode --tests` passes.
* The task is archived and the session is recorded after commit.

## Out of Scope

* Implementing a new HLS feature such as full subtitle burn-in, multi-variant
  ABR policy expansion, or remote worker execution.
* Rewriting the whole FFmpeg adapter.
* Changing playback planner selection semantics.
* Changing server HLS lifecycle orchestration.

## Technical Approach

Start with a narrow refactor in `crates/nako-transcode/src/ffmpeg/hls.rs` and
its HLS submodules. Prefer consolidating existing command-part assembly or
output-group construction over adding new behavior. The preferred shape is a
behavior-preserving typed builder seam that makes future subtitles/audio/HDR
extensions harder to scatter across callers.

## Decision (ADR-lite)

**Context**: HLS command planning already has useful submodules, but future
Jellyfin-grade parity will increase pressure on sidecar outputs, adaptive
variant outputs, filters, and muxer flags.

**Decision**: Deepen the existing HLS FFmpeg builder seam with a bounded
behavior-preserving refactor and exact argv coverage.

**Consequences**: The FFmpeg adapter remains the only command assembly
authority. Playback planning, transcode profile identity, artifact manifests,
and server orchestration stay in their existing boundaries.

## Technical Notes

* Files inspected:
  * `crates/nako-transcode/src/ffmpeg/hls.rs`
  * `crates/nako-transcode/src/ffmpeg/hls/encoders.rs`
  * `crates/nako-transcode/src/ffmpeg/hls/filters.rs`
  * `crates/nako-transcode/src/ffmpeg/hls/input.rs`
  * `crates/nako-transcode/src/ffmpeg/hls/muxer.rs`
  * `crates/nako-transcode/src/ffmpeg/hls/sidecars.rs`
  * `.trellis/spec/nako-transcode/backend/quality-guidelines.md`
  * `.trellis/tasks/archive/2026-06/06-03-transcode-architecture-parity-jellyfin/research/jellyfin-transcode-parity.md`

## Verification

* `cargo clippy -p nako-transcode --tests -- -D warnings`
* `cargo fmt --all -- --check`
* `git diff --check`
* `cargo check -p nako-core --tests`
* `cargo check -p nako-transcode --tests`
* `cargo nextest run -p nako-transcode hls --no-fail-fast`
* `cargo nextest run -p nako-transcode ffmpeg_builder --no-fail-fast`
