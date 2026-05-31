# Transcode Capability Inventory Matrix - TODO

Status: Active
Last updated: 2026-05-31

## M0 - Scope And Evidence Freeze

- [x] TCIM-010 [owner=planner] [deps=none] [scope=docs/workstreams/transcode-capability-inventory-matrix,docs/architecture/PLAYBACK.md,docs/architecture/WORKSTREAM_LINKS.md,docs/architecture/LANES.md,docs/workstreams/README.md]
  Goal: Open a transcode-only capability inventory lane that can run beside HDR `HTP-030` without touching pipeline selection or FFmpeg command execution.
  Validation: `python -m json.tool docs/workstreams/transcode-capability-inventory-matrix/WORKSTREAM.json`; `git diff --check -- docs/workstreams/transcode-capability-inventory-matrix docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md docs/architecture/LANES.md docs/workstreams/README.md`
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, and planner closeout note.
  Context: `docs/workstreams/transcode-capability-inventory-matrix/CONTEXT.jsonl`.
  Handoff: DONE. First executable task is `TCIM-020`.

## M1 - Inventory Matrix Facts

- [ ] TCIM-020 [owner=codex] [deps=TCIM-010] [scope=crates/nako-transcode/src/hardware.rs,crates/nako-transcode/src/probe.rs,crates/nako-transcode/src/lib.rs]
  Goal: Extend transcode capability inventory/report tests and values so decoder, encoder, filter, tone-map, subtitle, and bitstream-filter facts can be represented without changing HLS pipeline selection or FFmpeg command planning.
  Validation: `cargo nextest run -p nako-transcode hardware --no-fail-fast`; `cargo nextest run -p nako-transcode probe --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Review: Use `review-workstream` before accepting completion.
  Evidence: capability inventory tests and `EVIDENCE_AND_GATES.md`.
  Context: `docs/workstreams/transcode-capability-inventory-matrix/CONTEXT.jsonl`.
  Handoff: Keep the task inside inventory/report seams. Stop if changes need `pipeline.rs`, `ffmpeg.rs`, server routes, Public Client DTOs, or release packaging.

## M2 - Closeout

- [ ] TCIM-030 [owner=planner] [deps=TCIM-020] [scope=docs/workstreams/transcode-capability-inventory-matrix,docs/architecture/PLAYBACK.md,docs/architecture/LANES.md,docs/workstreams/README.md]
  Goal: Verify final gates, record evidence, and close or split follow-ons.
  Validation: final gates from `EVIDENCE_AND_GATES.md`; `python -m json.tool docs/workstreams/transcode-capability-inventory-matrix/WORKSTREAM.json`; `git diff --check`
  Review: `review-workstream` and `verify-rust-workstream` before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and closeout notes.
  Handoff: Split pipeline selection, HDR filter execution, HEVC/AV1 output, subtitle burn-in, and release packaging into follow-ons.
