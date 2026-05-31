# Playback Compatibility Matrix Hardening - TODO

Status: Active
Last updated: 2026-05-31

## M0 - Scope And Evidence Freeze

- [x] PCMH-010 [owner=planner] [deps=none] [scope=docs/workstreams/playback-compatibility-matrix-hardening,docs/architecture/PLAYBACK.md,docs/architecture/WORKSTREAM_LINKS.md,docs/architecture/LANES.md,docs/workstreams/README.md]
  Goal: Open the playback-only compatibility matrix lane and prove it can run beside HDR `HTP-030`.
  Validation: `python -m json.tool docs/workstreams/playback-compatibility-matrix-hardening/WORKSTREAM.json`; `git diff --check -- docs/workstreams/playback-compatibility-matrix-hardening docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md docs/architecture/LANES.md docs/workstreams/README.md`
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, and planner closeout note.
  Context: `docs/workstreams/playback-compatibility-matrix-hardening/CONTEXT.jsonl`.
  Handoff: DONE. First executable task is `PCMH-020`.

## M1 - Playback Decision Matrix

- [ ] PCMH-020 [owner=codex] [deps=PCMH-010] [scope=crates/nako-playback/src/lib.rs,crates/nako-playback/src/capability.rs,crates/nako-playback/src/values.rs]
  Goal: Add a table-driven playback compatibility matrix covering Direct Play, Remux, and HLS Transcode decisions across representative container, codec, HDR, audio channel, downmix, and normalization inputs.
  Validation: `cargo nextest run -p nako-playback compatibility --no-fail-fast`; `cargo nextest run -p nako-playback hdr audio --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Review: Use `review-workstream` before accepting completion.
  Evidence: matrix tests and `EVIDENCE_AND_GATES.md`.
  Context: `docs/workstreams/playback-compatibility-matrix-hardening/CONTEXT.jsonl`.
  Handoff: Keep fixes playback-only. Stop if the task requires `nako-transcode`, `nako-server`, Public Client DTOs, persisted preferences, or device profile databases.

## M2 - Closeout

- [ ] PCMH-030 [owner=planner] [deps=PCMH-020] [scope=docs/workstreams/playback-compatibility-matrix-hardening,docs/architecture/PLAYBACK.md,docs/architecture/LANES.md,docs/workstreams/README.md]
  Goal: Verify final gates, record evidence, and close the playback matrix lane.
  Validation: final gates from `EVIDENCE_AND_GATES.md`; `python -m json.tool docs/workstreams/playback-compatibility-matrix-hardening/WORKSTREAM.json`; `git diff --check`
  Review: `review-workstream` and `verify-rust-workstream` before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and closeout notes.
  Handoff: Split any discovered transcode/server/API gaps into follow-ons instead of expanding this crate-local lane.
