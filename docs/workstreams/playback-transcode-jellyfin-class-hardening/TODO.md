# Playback Transcode Jellyfin-Class Hardening - TODO

Status: Active
Last updated: 2026-05-31

## M0 - Interface And Parallel Lane Freeze

- [x] PTJCH-010 [owner=planner] [deps=none] [scope=docs/workstreams/playback-transcode-jellyfin-class-hardening,docs/architecture]
  Goal: Freeze the playback/transcode seam map, owned scopes, shared scopes,
  task order, stop conditions, and validation gates for parallel Codex work.
  Validation: `python -m json.tool docs/workstreams/playback-transcode-jellyfin-class-hardening/WORKSTREAM.json`; `git diff --check -- docs/workstreams/playback-transcode-jellyfin-class-hardening docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/architecture/PLAYBACK.md docs/workstreams/README.md`
  Review: Planner review before any Rust implementation begins.
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`, and updated architecture
  links.
  Context: `CONTEXT.jsonl`.
  Handoff: DONE. Opened the workstream, recorded the seam map, owned scopes,
  shared scopes, stop conditions, architecture links, and docs/planning gates.

- [x] PTJCH-020 [owner=planner] [deps=PTJCH-010] [scope=docs/workstreams/playback-transcode-jellyfin-class-hardening]
  Goal: Prepare first-batch worker prompts and branch/worktree guidance for
  Playback Capability, Transcode Pipeline Capability, and FFmpeg Adapter lanes.
  Validation: `python -m json.tool docs/workstreams/playback-transcode-jellyfin-class-hardening/WORKSTREAM.json`; `git diff --check -- docs/workstreams/playback-transcode-jellyfin-class-hardening`
  Review: Ensure prompts list owned scopes, shared scopes, stop conditions,
  and task-local gates.
  Evidence: `HANDOFF.md` and this task ledger.
  Context: `CONTEXT.jsonl`.
  Handoff: DONE. Added `WORKER_PROMPTS.md`, suggested worktree branches, and
  task-local worker note paths for the first parallel batch.

## M1 - First Parallel Batch

- [x] PTJCH-110 [owner=codex] [deps=PTJCH-020] [scope=crates/nako-playback]
  Goal: Deepen Playback Capability conditions and compatibility reason tests
  without adding transcode execution or server lifecycle behavior.
  Validation: `cargo nextest run -p nako-playback --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Review: Use `review-workstream` before accepting completion.
  Evidence: focused playback planner tests and `EVIDENCE_AND_GATES.md`.
  Context: `CONTEXT.jsonl`.
  Handoff: DONE. Merged commit `0d3bd96f`; remux capability evaluation now
  preserves playback output constraints and transcode requirements carry remux
  blocker reasons.

- [x] PTJCH-120 [owner=codex] [deps=PTJCH-020] [scope=crates/nako-transcode/src/pipeline.rs,crates/nako-transcode/src/hardware.rs,crates/nako-transcode/src/probe.rs]
  Goal: Deepen stage-aware Transcode Pipeline Capability matching for hardware
  and fallback requirements without changing server runtime behavior.
  Validation: `cargo nextest run -p nako-transcode pipeline hardware probe --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Review: Use `review-workstream` before accepting completion.
  Evidence: pipeline/hardware/probe tests and `EVIDENCE_AND_GATES.md`.
  Context: `CONTEXT.jsonl`.
  Handoff: DONE. Merged commit `9f841951`; pipeline planning now checks
  requested decode-stage support for source codec compatibility.

- [x] PTJCH-130 [owner=codex] [deps=PTJCH-020] [scope=crates/nako-transcode/src/ffmpeg.rs,crates/nako-transcode/src/execution.rs]
  Goal: Split FFmpeg Adapter internals so command planning breadth can grow
  without creating one large encoding helper.
  Validation: `cargo nextest run -p nako-transcode ffmpeg hls --no-fail-fast`; `cargo nextest run -p nako-transcode remux --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Review: Use `review-workstream` before accepting completion.
  Evidence: command planning tests and `EVIDENCE_AND_GATES.md`.
  Context: `CONTEXT.jsonl`.
  Handoff: DONE. Merged commit `bb3835e0`; FFmpeg command planning internals
  are split behind the existing builder facade.

## M2 - Coordinated HLS Runtime And Artifact Batch

- [ ] PTJCH-210 [owner=codex] [deps=PTJCH-110,PTJCH-120,PTJCH-130] [scope=crates/nako-transcode/src/artifact.rs,crates/nako-server/src/app/playback/staging_policy.rs]
  Goal: Freeze or deepen HLS Artifact Authority around request variant
  identity, manifest reconstruction, and serveable artifact allow-lists.
  Validation: `cargo nextest run -p nako-transcode hls --no-fail-fast`; `cargo nextest run -p nako-server hls --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Review: Planner coordination required because this touches server playback
  integration.
  Evidence: artifact identity/reconstruction tests and `EVIDENCE_AND_GATES.md`.
  Context: `CONTEXT.jsonl`.
  Handoff: Pending.

- [ ] PTJCH-220 [owner=codex] [deps=PTJCH-110,PTJCH-120,PTJCH-130] [scope=crates/nako-server/src/app/playback]
  Goal: Clarify Playback Runtime ownership for sessions, admission, reuse,
  supersede, cancel, failure classification, and diagnostics without moving
  transcode planning back into the server.
  Validation: `cargo nextest run -p nako-server hls playback --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Review: Planner coordination required if `PTJCH-210` is active.
  Evidence: runtime lifecycle tests and `EVIDENCE_AND_GATES.md`.
  Context: `CONTEXT.jsonl`.
  Handoff: Pending.

## M3 - Artifact I/O And Closeout

- [ ] PTJCH-310 [owner=planner] [deps=PTJCH-210,PTJCH-220] [scope=docs/workstreams/playback-transcode-jellyfin-class-hardening,docs/architecture]
  Goal: Decide whether HLS artifact I/O pressure belongs inside this workstream
  or should split to a dedicated PAIP follow-on.
  Validation: `python -m json.tool docs/workstreams/playback-transcode-jellyfin-class-hardening/WORKSTREAM.json`; `git diff --check`
  Review: Planner decision required.
  Evidence: `EVIDENCE_AND_GATES.md` and updated follow-on links.
  Context: `CONTEXT.jsonl`.
  Handoff: Pending.

- [ ] PTJCH-390 [owner=planner] [deps=PTJCH-310] [scope=docs/workstreams/playback-transcode-jellyfin-class-hardening,docs/architecture]
  Goal: Close this coordination workstream or split remaining implementation
  into dedicated workstreams with explicit owners and gates.
  Validation: final gates from `EVIDENCE_AND_GATES.md`; `python -m json.tool docs/workstreams/playback-transcode-jellyfin-class-hardening/WORKSTREAM.json`; `git diff --check`
  Review: Use `review-workstream` and `verify-rust-workstream` before closeout
  if Rust code changed.
  Evidence: closeout notes and final gate output.
  Context: `CONTEXT.jsonl`.
  Handoff: Pending.
