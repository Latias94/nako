# HLS Selected Main Audio Cleanup - Evidence And Gates

Status: Completed
Last updated: 2026-05-29

## Smallest Current Repro

The current proof is the closed HSMA-030/040 output-shape cleanup.

```bash
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

## Gate Set

### Planning Gate

```bash
python3 -m json.tool docs/workstreams/hls-selected-main-audio-cleanup/WORKSTREAM.json
git diff --check -- docs/workstreams/hls-selected-main-audio-cleanup docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md
```

Proves workstream docs are syntactically valid and whitespace-clean.

### Transcode HLS Gate

```bash
cargo nextest run -p nako-transcode hls --no-fail-fast
```

Proves FFmpeg command planning, HLS artifact identity, and HLS output-shape
unit coverage.

### Server HLS Gate

```bash
cargo nextest run -p nako-server hls --no-fail-fast
```

Proves server-side HLS planning, playlist authoring, artifact serving, and
ticketed route behavior for focused HLS tests.

### Playback Regression Gate

```bash
cargo nextest run -p nako-server playback --no-fail-fast
```

Proves the HLS cleanup does not regress broader playback routes, runtime
sessions, browser tickets, renderer tickets, session reuse, or Admin playback
diagnostics covered by the playback filter.

### Final Closeout Gate

```bash
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo fmt --all -- --check
python3 -m json.tool docs/workstreams/hls-selected-main-audio-cleanup/WORKSTREAM.json
git diff --check
```

Use broader workspace gates only if this lane changes public API contracts or
shared crate boundaries outside HLS command/output shape.

### Review Gate

Run `review-workstream` before accepting task or lane completion. Record
blocking findings, missing gates, and residual risks here or link to a review
note.

## Evidence Log

| Date | Task | Evidence | Status | Notes |
| --- | --- | --- | --- | --- |
| 2026-05-29 | HSMA-010 | Workstream opened | Passed | Fresh gates: `python3 -m json.tool docs/workstreams/hls-selected-main-audio-cleanup/WORKSTREAM.json`; `git diff --check -- docs/workstreams/hls-selected-main-audio-cleanup docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md`. |
| 2026-05-29 | HSMA-020 | Duplicated selected audio characterization | Passed | Added characterization coverage for single-variant and adaptive sidecar-capable main mux duplication, then HSMA-030 converted those tests to the shipped no-duplication expectation. Fresh gates at characterization time: `cargo nextest run -p nako-transcode hls --no-fail-fast` (42 passed, 38 skipped); `cargo nextest run -p nako-server hls --no-fail-fast` (53 passed, 422 skipped). `review-workstream` self-review found no blocking findings; HSMA-030 owns behavior change. |
| 2026-05-29 | HSMA-030 | Selected main mux cleanup | Passed | Added `HlsArtifactManifest::main_output_has_audio()`, updated single-variant/adaptive FFmpeg HLS main-output map/encoder/`-var_stream_map` planning to omit selected audio when generated audio sidecars exist, and added `hls-main-output:v1;main_audio=false` to request variant identity. Fresh gates: `cargo nextest run -p nako-transcode hls --no-fail-fast` (42 passed, 38 skipped); `cargo nextest run -p nako-server hls --no-fail-fast` (53 passed, 422 skipped); `cargo nextest run -p nako-server playback --no-fail-fast` (132 passed, 343 skipped). An initial transcode HLS run hit a transient `hls_runner_can_publish_output_while_process_is_running` timing failure; immediate targeted rerun and fresh full rerun passed. |
| 2026-05-29 | HSMA-040 | Closeout and follow-on split | Passed | Closeout docs mark the lane complete and defer language preferences, codec-aware sidecars, LL-HLS/DASH/DRM, and player-specific fallback. Final gates: `cargo fmt --all -- --check`; `python3 -m json.tool docs/workstreams/hls-selected-main-audio-cleanup/WORKSTREAM.json`; `git diff --check`. `review-workstream`/`verify-rust-workstream` closeout review found no blocking or important findings. |

## Evidence Anchors

- `docs/workstreams/hls-selected-main-audio-cleanup/DESIGN.md`
- `docs/workstreams/hls-selected-main-audio-cleanup/TODO.md`
- `docs/workstreams/hls-selected-main-audio-cleanup/MILESTONES.md`
- `crates/nako-transcode/src/ffmpeg.rs`
- `crates/nako-transcode/src/artifact.rs`
- `crates/nako-transcode/src/lib.rs`
- `crates/nako-server/src/app/playback`
- `docs/workstreams/hls-selected-main-audio-cleanup/CLOSEOUT.md`

Fresh verification is required before marking a task, Codex goal, or lane
complete.
