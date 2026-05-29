# HLS Selected Main Audio Cleanup - TODO

Status: Completed
Last updated: 2026-05-29

## Task Ledger

### HSMA-010 - Open lane and freeze cleanup boundary

Status: Completed
Owner: codex
Depends on: none

Scope:

- `docs/workstreams/hls-selected-main-audio-cleanup`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/workstreams/README.md`

Goal:

- Create the durable workstream.
- Freeze selected-main-audio duplication removal as a cleanup after generated
  HLS audio sidecar artifacts.
- Keep language policy, codec-aware sidecars, LL-HLS/DASH/DRM, and
  player-specific fallback out of this lane.

Validation:

```text
python3 -m json.tool docs/workstreams/hls-selected-main-audio-cleanup/WORKSTREAM.json
git diff --check -- docs/workstreams/hls-selected-main-audio-cleanup docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md
```

Evidence:

- `docs/workstreams/hls-selected-main-audio-cleanup/DESIGN.md`
- `docs/workstreams/hls-selected-main-audio-cleanup/EVIDENCE_AND_GATES.md`

Handoff:

- DONE: This lane is open and linked from playback architecture indexes.
- The first executable task is HSMA-020.

### HSMA-020 - Characterize current duplicated audio output shape

Status: Completed
Owner: codex
Depends on: HSMA-010

Scope:

- `crates/nako-transcode/src/ffmpeg.rs`
- `crates/nako-transcode/src/lib.rs`
- `crates/nako-server/src/app/playback`

Goal:

- Add or update focused tests that prove the current duplicated selected-main
  audio behavior for multi-audio sidecar-capable HLS outputs.
- Preserve single-audio and no-audio expectations before changing command
  planning.
- Identify the smallest command/request-shape change needed to remove the
  duplication.

Validation:

```text
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
```

Review:

- Use `review-workstream` before accepting completion.

Evidence:

- `crates/nako-transcode/src/lib.rs`
- `crates/nako-server/src/app/playback`
- `docs/workstreams/hls-selected-main-audio-cleanup/EVIDENCE_AND_GATES.md`

Handoff:

- DONE: Added transcode characterization tests for single-variant and adaptive
  HLS outputs that currently duplicate selected audio when generated audio
  sidecars exist.
- DONE: The smallest implementation surface is FFmpeg HLS main-output stream
  mapping, adaptive audio encoder/muxer stream-map construction, and the server
  request/artifact facts needed to decide whether generated audio sidecars are
  present.
- HSMA-030 should convert those characterization tests into the new expected
  output shape.

### HSMA-030 - Remove selected audio from sidecar-capable main mux

Status: Completed
Owner: codex
Depends on: HSMA-020

Scope:

- `crates/nako-transcode/src/ffmpeg.rs`
- `crates/nako-transcode/src/artifact.rs` if request identity needs to encode
  main-output audio shape
- `crates/nako-server/src/app/playback`
- Focused HLS/playback tests

Goal:

- Make sidecar-capable multi-audio HLS main video outputs avoid muxing the
  selected audio stream.
- Keep generated audio sidecars as the source of advertised audio groups.
- Preserve single-audio, no-audio, and no-sidecar output behavior.
- Preserve public browser/renderer HLS route contracts.

Validation:

```text
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

Review:

- Use `review-workstream` before accepting completion.

Evidence:

- Updated HLS command planning tests.
- Updated server HLS integration tests.
- `docs/workstreams/hls-selected-main-audio-cleanup/EVIDENCE_AND_GATES.md`

Handoff:

- DONE: Sidecar-capable single-variant and adaptive HLS main outputs no longer
  mux the selected audio stream.
- DONE: Single-audio and no-sidecar output behavior remains covered by existing
  HLS tests.
- DONE: HLS request variant identity includes the main-output audio shape for
  generated audio sidecar outputs.
- DONE: Player-specific fallback remains out of scope.

### HSMA-040 - Verify, document, and close or split follow-ons

Status: Completed
Owner: codex
Depends on: HSMA-030

Scope:

- `docs/workstreams/hls-selected-main-audio-cleanup`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`

Goal:

- Run fresh closeout gates.
- Record final evidence and residual risks.
- Close the lane or split follow-ons for language preference policy,
  codec-aware sidecars, LL-HLS/DASH/DRM, or player-specific fallback.

Validation:

```text
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Review:

- Use `review-workstream` and `verify-rust-workstream` before closeout.

Evidence:

- `docs/workstreams/hls-selected-main-audio-cleanup/EVIDENCE_AND_GATES.md`
- `docs/workstreams/hls-selected-main-audio-cleanup/HANDOFF.md`
- `docs/workstreams/hls-selected-main-audio-cleanup/WORKSTREAM.json`

Handoff:

- DONE: `WORKSTREAM.json` status and continue policy were updated.
- DONE: `CLOSEOUT.md`, `EVIDENCE_AND_GATES.md`, and architecture docs record
  the shipped output shape.
- DONE: Language preferences, codec-aware sidecars, LL-HLS/DASH/DRM, and
  player-specific fallback remain deferred follow-ons.
