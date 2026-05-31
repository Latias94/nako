# HLS Runtime Lifecycle Boundary - TODO

Status: Active
Last updated: 2026-05-31

## M0 - Lifecycle Invariant Freeze

- [x] HRLB-010 [owner=planner] [deps=none] [scope=docs/workstreams/hls-runtime-lifecycle-boundary,docs/architecture/PLAYBACK.md,docs/architecture/LANES.md,docs/architecture/WORKSTREAM_LINKS.md,docs/workstreams/README.md]
  Goal: Freeze current HLS lifecycle invariants, readiness semantics, cleanup ownership, and test coverage map before any runtime behavior change.
  Validation: `python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json`; `git diff --check -- docs/workstreams/hls-runtime-lifecycle-boundary docs/architecture/PLAYBACK.md docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md`
  Review: Planner review before implementation tasks start.
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`, lifecycle invariant table, cleanup ownership map, PAIP split decision, and test coverage map.
  Context: `docs/workstreams/hls-runtime-lifecycle-boundary/CONTEXT.jsonl`.
  Handoff: DONE_WITH_CONCERNS. No Rust behavior changed. `HRLB-020` should add focused tests for HLS timeout cleanup, HLS startup stale recovery, and HLS remote staging input release before or during any coordinator extraction.

## M1 - Behavior-Preserving Lifecycle Boundary

- [x] HRLB-020 [owner=codex] [deps=HRLB-010] [scope=crates/nako-server/src/app/playback/hls.rs,crates/nako-server/src/app/playback/hls_artifact.rs,crates/nako-server/src/app/playback/resource.rs,crates/nako-server/src/app/playback/control.rs,crates/nako-server/src/app/playback/support.rs,crates/nako-server/src/app/tests/playback.rs]
  Goal: Add focused invariant tests and, if justified by `HRLB-010`, introduce a behavior-preserving server-local lifecycle coordinator/facade.
  Validation: `cargo nextest run -p nako-server hls --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Review: Use `review-workstream` before accepting completion.
  Evidence: server HLS lifecycle tests and `EVIDENCE_AND_GATES.md`.
  Context: `docs/workstreams/hls-runtime-lifecycle-boundary/CONTEXT.jsonl`.
  Handoff: DONE_WITH_CONCERNS. Added focused tests in `crates/nako-server/src/app/tests/playback.rs` for HLS timeout cleanup, HLS stale startup recovery, and HLS remote staged-input release across success, runner error, and admission rejection. Did not introduce a lifecycle coordinator/facade because HRLB-010 justified coverage first, not a new abstraction. Planner accepted a narrow scope-out Admin DTO stage mapping fix for `HardwarePipelineStage::{ToneMap, SubtitleBurnIn}`. Final HLS gate passed after a load-sensitive progressive-readiness test passed individually and on full rerun.

## M2 - Follow-On Split

- [x] HRLB-030 [owner=planner] [deps=HRLB-020] [scope=docs/workstreams/hls-runtime-lifecycle-boundary,docs/architecture/PLAYBACK.md,docs/architecture/STORAGE_VFS.md,docs/architecture/LANES.md,docs/architecture/WORKSTREAM_LINKS.md]
  Goal: Decide whether artifact I/O pressure, resource admission unification, remote workers, LL-HLS/CMAF, player UX, and HLS test stability remain follow-ons or become the next bounded workstream.
  Validation: `python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json`; `git diff --check`
  Review: Planner review before opening follow-on workstreams.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and follow-on decision notes.
  Handoff: DONE_WITH_CONCERNS. Split PAIP artifact I/O pressure, resource admission unification, remote workers, LL-HLS/CMAF, and player UX into separate follow-ons. Recommend `proposed:hls-progressive-readiness-test-stability` as the next bounded workstream before PAIP because HRLB-020 left load-sensitive HLS gate evidence despite final validation passing.

## M3 - Closeout

- [ ] HRLB-040 [owner=planner] [deps=HRLB-030] [scope=docs/workstreams/hls-runtime-lifecycle-boundary,docs/architecture/PLAYBACK.md,docs/architecture/LANES.md,docs/workstreams/README.md]
  Goal: Verify final gates, record evidence, and close or split remaining follow-ons.
  Validation: final gates from `EVIDENCE_AND_GATES.md`; `python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json`; `git diff --check`
  Review: `review-workstream` and `verify-rust-workstream` before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and closeout notes.
  Handoff: Do not leave lifecycle/admission ownership decisions only in chat.
