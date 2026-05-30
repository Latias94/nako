# Transcode Interface And Runtime Plan Deepening - TODO

Status: Active
Last updated: 2026-05-31

## M0 - Scope And Evidence Freeze

- [x] TIRP-010 [owner=planner] [deps=none] [scope=docs/workstreams/transcode-interface-and-runtime-plan-deepening,docs/architecture]
  Goal: Freeze the transcode Interface deepening target before HDR tone mapping resumes.
  Validation: `python -m json.tool docs/workstreams/transcode-interface-and-runtime-plan-deepening/WORKSTREAM.json`; `git diff --check -- docs/workstreams/transcode-interface-and-runtime-plan-deepening docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md docs/architecture/LANES.md docs/workstreams/README.md`
  Evidence: `docs/workstreams/transcode-interface-and-runtime-plan-deepening/DESIGN.md`
  Context: `docs/workstreams/transcode-interface-and-runtime-plan-deepening/CONTEXT.jsonl`
  Handoff: DONE. Planner opened the lane and made `TIRP-020` the first executable task.

## M1 - HLS Runtime Plan Interface

- [ ] TIRP-020 [owner=codex] [deps=TIRP-010] [scope=crates/nako-transcode/src/pipeline.rs,crates/nako-transcode/src/profile.rs,crates/nako-transcode/src/artifact.rs,crates/nako-transcode/src/lib.rs,crates/nako-server/src/app/playback/mod.rs,crates/nako-server/src/app/playback/staging_policy.rs]
  Goal: Introduce a transcode-owned HLS runtime plan Interface and move the HLS profile/request-variant/execution-policy assembly out of server orchestration without changing runtime behavior.
  Validation: `cargo nextest run -p nako-transcode hls audio --no-fail-fast`; `cargo nextest run -p nako-server hls --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Review: Use `review-workstream` before accepting completion.
  Evidence: pure transcode planning tests, focused server HLS regression tests, and `EVIDENCE_AND_GATES.md`.
  Context: `docs/workstreams/transcode-interface-and-runtime-plan-deepening/CONTEXT.jsonl`.
  Handoff: Keep HDR tone mapping, hardware capability matrix expansion, HLS lifecycle consolidation, and resource admission unification out of this task.

## M2 - FFmpeg Adapter Interface Ratchet

- [ ] TIRP-030 [owner=codex] [deps=TIRP-020] [scope=crates/nako-transcode/src/lib.rs,crates/nako-transcode/src/ffmpeg.rs,crates/nako-transcode/src/execution.rs,crates/nako-transcode/src/hls.rs,crates/nako-transcode/src/remux.rs,crates/nako-server/src/app/playback/hls.rs,crates/nako-server/src/app/playback/remux.rs]
  Goal: Curate `nako-transcode` exports and keep low-level FFmpeg request/builder details behind internal execution adapters while preserving server HLS/remux behavior.
  Validation: `cargo nextest run -p nako-transcode hls --no-fail-fast`; `cargo nextest run -p nako-transcode remux --no-fail-fast`; `cargo nextest run -p nako-server hls --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Review: Use `review-workstream` before accepting completion.
  Evidence: compile-time server import reduction, transcode command-plan tests, HLS/remux runtime tests, and `EVIDENCE_AND_GATES.md`.
  Context: `docs/workstreams/transcode-interface-and-runtime-plan-deepening/CONTEXT.jsonl`.
  Handoff: If server still needs raw `HlsRequest`, `RemuxRequest`, `FfmpegCommandBuilder`, or `FfmpegArg`, return to planner instead of widening `pub use`.

## M3 - Closeout And HDR Unblock

- [ ] TIRP-040 [owner=planner] [deps=TIRP-030] [scope=docs/workstreams/transcode-interface-and-runtime-plan-deepening,docs/architecture/PLAYBACK.md,docs/architecture/WORKSTREAM_LINKS.md,docs/architecture/LANES.md,docs/workstreams/README.md]
  Goal: Verify final gates, close this lane, and explicitly unblock or replan HDR `HTP-030`.
  Validation: `python -m json.tool docs/workstreams/transcode-interface-and-runtime-plan-deepening/WORKSTREAM.json`; final gates from `EVIDENCE_AND_GATES.md`; `git diff --check`
  Review: `review-workstream` and `verify-rust-workstream` before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and closeout notes.
  Handoff: Split stage-aware HDR/tone-map capability matrix, HLS lifecycle consolidation, and resource admission unification into follow-ons.
