# HLS Runtime Lifecycle Boundary - TODO

Status: Active
Last updated: 2026-05-31

## M0 - Lifecycle Invariant Freeze

- [ ] HRLB-010 [owner=planner] [deps=none] [scope=docs/workstreams/hls-runtime-lifecycle-boundary,docs/architecture/PLAYBACK.md,docs/architecture/LANES.md,docs/architecture/WORKSTREAM_LINKS.md,docs/workstreams/README.md]
  Goal: Freeze current HLS lifecycle invariants, readiness semantics, cleanup ownership, and test coverage map before any runtime behavior change.
  Validation: `python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json`; `git diff --check -- docs/workstreams/hls-runtime-lifecycle-boundary docs/architecture/PLAYBACK.md docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md`
  Review: Planner review before implementation tasks start.
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`, lifecycle invariant table, and test coverage map.
  Context: `docs/workstreams/hls-runtime-lifecycle-boundary/CONTEXT.jsonl`.
  Handoff: Do not edit Rust behavior in this task.

## M1 - Behavior-Preserving Lifecycle Boundary

- [ ] HRLB-020 [owner=codex] [deps=HRLB-010] [scope=crates/nako-server/src/app/playback/hls.rs,crates/nako-server/src/app/playback/hls_artifact.rs,crates/nako-server/src/app/playback/resource.rs,crates/nako-server/src/app/playback/control.rs,crates/nako-server/src/app/playback/support.rs,crates/nako-server/src/app/tests/playback.rs]
  Goal: Add focused invariant tests and, if justified by `HRLB-010`, introduce a behavior-preserving server-local lifecycle coordinator/facade.
  Validation: `cargo nextest run -p nako-server hls --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Review: Use `review-workstream` before accepting completion.
  Evidence: server HLS lifecycle tests and `EVIDENCE_AND_GATES.md`.
  Context: `docs/workstreams/hls-runtime-lifecycle-boundary/CONTEXT.jsonl`.
  Handoff: Stop if the task needs `nako-transcode` command planning, pipeline selection, API DTOs, storage schema, or release packaging.

## M2 - Follow-On Split

- [ ] HRLB-030 [owner=planner] [deps=HRLB-020] [scope=docs/workstreams/hls-runtime-lifecycle-boundary,docs/architecture/PLAYBACK.md,docs/architecture/STORAGE_VFS.md,docs/architecture/LANES.md,docs/architecture/WORKSTREAM_LINKS.md]
  Goal: Decide whether artifact I/O pressure, resource admission unification, remote workers, LL-HLS/CMAF, and player UX remain follow-ons or become the next bounded workstream.
  Validation: `python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json`; `git diff --check`
  Review: Planner review before opening follow-on workstreams.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and follow-on decision notes.
  Handoff: Split storage artifact I/O pressure if it needs storage health/circuit-breaker or Admin diagnostics scope.

## M3 - Closeout

- [ ] HRLB-040 [owner=planner] [deps=HRLB-030] [scope=docs/workstreams/hls-runtime-lifecycle-boundary,docs/architecture/PLAYBACK.md,docs/architecture/LANES.md,docs/workstreams/README.md]
  Goal: Verify final gates, record evidence, and close or split remaining follow-ons.
  Validation: final gates from `EVIDENCE_AND_GATES.md`; `python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json`; `git diff --check`
  Review: `review-workstream` and `verify-rust-workstream` before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and closeout notes.
  Handoff: Do not leave lifecycle/admission ownership decisions only in chat.
