# CPU Transcode Readiness - TODO

Status: Complete
Last updated: 2026-05-27

## M0 - Workstream Open

- [x] CTR-010 [owner=codex] [deps=none] [scope=docs/workstreams/cpu-transcode-readiness,docs/adr]
  Goal: Open the lane and record ADR 0047.
  Validation: `python -m json.tool docs/workstreams/cpu-transcode-readiness/WORKSTREAM.json`.
  Evidence: ADR 0047 and initial workstream docs.
  Handoff: CTR-020 maps CPU readiness from probe inventory.

## M1 - CPU Capability Mapping

- [x] CTR-020 [owner=codex] [deps=CTR-010] [scope=crates/nako-transcode/src/hardware.rs,crates/nako-transcode/src/probe.rs,crates/nako-transcode/src/lib.rs]
  Goal: Build probe-derived CPU capability from required `libx264` and `aac`
  encoder facts.
  Validation: `cargo nextest run -p nako-transcode cpu --no-fail-fast`.
  Evidence: `cargo nextest run -p nako-transcode --no-fail-fast`.
  Review: static `cpu_only()` fixtures remain explicit test fixtures, while
  probe-derived reports require software encoder facts.
  Handoff: CTR-030 completed pipeline fallback semantics.

## M2 - Pipeline Fallback Semantics

- [x] CTR-030 [owner=codex] [deps=CTR-020] [scope=crates/nako-transcode/src/pipeline.rs,crates/nako-api/src/admin/playback.rs,crates/nako-server/src/http/admin.rs]
  Goal: Reject explicit CPU planning and hardware fallback-to-CPU when the CPU
  software pipeline is unavailable; expose typed Admin readiness reasons.
  Validation: `cargo nextest run -p nako-transcode pipeline --no-fail-fast`;
  `cargo nextest run -p nako-api --no-fail-fast`.
  Evidence: `cargo nextest run -p nako-transcode --no-fail-fast`;
  `cargo nextest run -p nako-api --no-fail-fast`.
  Review: fail policy and unavailable CPU fallback remain distinguishable typed
  states.
  Handoff: CTR-040 completed server fake FFmpeg updates.

## M3 - Tests And Closeout

- [x] CTR-040 [owner=codex] [deps=CTR-030] [scope=crates/nako-server/src/app/tests,crates/nako-server/src/http/tests,docs/workstreams/cpu-transcode-readiness]
  Goal: Update fake FFmpeg scripts and focused tests, run final gates, and
  close the lane.
  Validation: `cargo nextest run -p nako-transcode --no-fail-fast`;
  `cargo nextest run -p nako-api --no-fail-fast`;
  `cargo nextest run -p nako-server playback --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`.
  Evidence: server playback focused gate passed; final format/diff gates are
  recorded in `EVIDENCE_AND_GATES.md`.
  Review: remaining work is split into follow-ons in `HANDOFF.md`.
