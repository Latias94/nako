# HLS text subtitle burn-in FFmpeg planning

## Goal

Make the HLS FFmpeg command planner accept `BurnInSelected` for embedded text
subtitles and emit a deterministic primary-video subtitle burn-in filter, while
keeping sidecar subtitle artifacts, playback selection, server routes, OCR, and
public API behavior out of this slice.

## What I already know

* The previous HLS subtitle strategy slice made playback select
  `SidecarSelected` or `BurnInSelected` explicitly and made the HLS runtime
  preserve that strategy.
* `crates/nako-transcode/src/ffmpeg/hls.rs` still rejects
  `TranscodeSubtitleStrategy::BurnInSelected`.
* `crates/nako-transcode/src/ffmpeg/hls/filters.rs` currently plans HDR/video
  filters and audio filters, but has no subtitle burn-in filter path.
* FFmpeg's `subtitles` filter can render a selected embedded text subtitle
  stream into the video path using the input filename and subtitle-stream
  ordinal.

## Requirements

* Accept `BurnInSelected` in the HLS FFmpeg command planner when a selected
  subtitle stream exists and no subtitle sidecar artifacts are present.
* Convert Nako's selected source stream index to FFmpeg's subtitle-stream
  ordinal before command planning.
* Derive the burn-in plan only from probe-confirmed embedded text subtitle
  streams; reject image subtitles, external subtitles, and missing codec facts.
* Emit the subtitle burn-in as a primary video filter, not as a sidecar output.
* Preserve existing sidecar behavior for `SidecarSelected`.
* Preserve existing rejection for `PreserveInContainer`.
* Reject `BurnInSelected` when no subtitle stream is selected.
* Reject subtitle sidecar artifacts unless the strategy is `SidecarSelected`.
* Keep executable burn-in logic inside `nako-transcode`; server changes may only
  pass the typed runtime plan into HLS execution.

## Acceptance Criteria

* [x] `FfmpegCommandBuilder::hls` plans a single-variant HLS command with
  `BurnInSelected`, `-vf`, and a
  `subtitles=...:si=<subtitle-ordinal>` filter.
* [x] The planned burn-in command maps video/audio only for the primary HLS
  output and does not emit subtitle sidecar segment outputs.
* [x] Existing sidecar subtitle command tests still pass.
* [x] `BurnInSelected` without a selected subtitle stream returns
  `NakoError::InvalidInput`.
* [x] `BurnInSelected` with image, external, or unknown-codec subtitle facts is
  rejected before FFmpeg command execution.
* [x] `PreserveInContainer` remains unsupported for HLS.
* [x] Focused `nako-transcode` HLS tests and cross-crate checks pass.

## Definition of Done

* Tests updated and passing.
* `cargo fmt --all -- --check` and `git diff --check` pass.
* `cargo nextest run -p nako-transcode hls --no-fail-fast` passes.
* `cargo check -p nako-transcode --tests` passes.
* Spec updated only if implementation introduces a durable new command-planning
  contract not already covered by the transcode quality guideline.
* Conventional Commit, task archive, and journal entry are recorded.

## Out of Scope

* OCR or image subtitle conversion.
* External subtitle download or staging.
* Public API, Admin API, Web UI, or playback route changes.
* Client/device subtitle capability expansion.
* Runtime probing of FFmpeg filter availability beyond existing inventory
  structures.

## Technical Approach

* Add a typed subtitle burn-in filter helper in the HLS filter module.
* Add a typed HLS subtitle burn-in plan that carries both Nako's selected source
  stream index and FFmpeg's subtitle-stream ordinal.
* Compose the subtitle filter with existing video filter planning so HDR
  tone-map and subtitle burn-in remain deterministic when both are present.
* Use exact argv tests in `crates/nako-transcode/src/lib.rs` to lock command
  shape.
* Keep HLS artifact manifests unchanged because burn-in is part of the primary
  output, not a separately served artifact.
* Pass the typed burn-in plan through server HLS execution without changing
  public routes or DTOs.

## Decision (ADR-lite)

**Context**: The runtime planner can now preserve `BurnInSelected`, but the
FFmpeg adapter still rejects it, so unsupported subtitle clients cannot progress
past planning.

**Decision**: Implement the first executable text-subtitle burn-in slice at the
FFmpeg command-planning layer. It will support embedded selected text subtitles
through the FFmpeg `subtitles` filter and leave image subtitles and external
subtitle sources for later tasks.

**Consequences**: The HLS command planner becomes the first executable consumer
of burn-in intent. Future codec-aware subtitle validation can tighten which
subtitle codecs are eligible without changing the playback/server strategy
boundary.

## Research References

* [`research/current-ffmpeg-hls-burn-in-builder.md`](research/current-ffmpeg-hls-burn-in-builder.md)
  - current local command builder state and bounded implementation seam.

## Technical Notes

* Relevant specs:
  * `.trellis/spec/nako-transcode/backend/index.md`
  * `.trellis/spec/nako-transcode/backend/quality-guidelines.md`
* Relevant local files:
  * `crates/nako-transcode/src/ffmpeg/hls.rs`
  * `crates/nako-transcode/src/ffmpeg/hls/filters.rs`
  * `crates/nako-transcode/src/ffmpeg/hls/input.rs`
  * `crates/nako-transcode/src/ffmpeg/hls/sidecars.rs`
  * `crates/nako-transcode/src/lib.rs`
* Prior task evidence:
  * `.trellis/tasks/archive/2026-06/06-04-playback-hls-subtitle-burn-in-planning/`
