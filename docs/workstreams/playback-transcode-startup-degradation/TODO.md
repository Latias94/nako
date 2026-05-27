# Playback Transcode Startup Degradation - TODO

Status: Complete
Last updated: 2026-05-27

## M0 - Workstream Open

- [x] PTSD-010 [owner=codex] [deps=none] [scope=docs/adr,docs/workstreams/playback-transcode-startup-degradation]
  Goal: Record ADR 0048 and open the execution lane.
  Validation: `python -m json.tool docs/workstreams/playback-transcode-startup-degradation/WORKSTREAM.json`.
  Evidence: ADR and workstream docs.
  Handoff: PTSD-020 splits HLS readiness from executable startup plan.

## M1 - Startup State Boundary

- [x] PTSD-020 [owner=codex] [deps=PTSD-010] [scope=crates/nako-server/src/app/playback/hls.rs,crates/nako-server/src/app/playback/mod.rs]
  Goal: Store HLS pipeline readiness separately from optional executable plan
  so startup can continue when transcode is unavailable.
  Validation: `cargo nextest run -p nako-server hls_service --no-fail-fast`.
  Evidence: `cargo nextest run -p nako-server hls_service --no-fail-fast`.
  Review: HLS execution still rejects unavailable planning before spawning
  FFmpeg.
  Handoff: PTSD-030 completed admin diagnostics semantics.

## M2 - Admin Diagnostics

- [x] PTSD-030 [owner=codex] [deps=PTSD-020] [scope=crates/nako-server/src/http/admin.rs,crates/nako-api/src/admin/playback.rs,crates/nako-server/src/http/tests/system.rs]
  Goal: Runtime diagnostics report unavailable HLS readiness and zero selected
  HLS slots when no executable HLS plan exists.
  Validation: `cargo nextest run -p nako-server admin_v1_playback_runtime --no-fail-fast`.
  Evidence: `cargo nextest run -p nako-server admin_v1_playback_runtime --no-fail-fast`;
  `cargo nextest run -p nako-server playback --no-fail-fast`.
  Review: selected fallback check does not claim ready when the pipeline is
  unavailable.
  Handoff: PTSD-040 completed focused gates and closeout.

## M3 - Gates And Closeout

- [x] PTSD-040 [owner=codex] [deps=PTSD-030] [scope=docs/workstreams/playback-transcode-startup-degradation]
  Goal: Run focused gates, update evidence, and close the lane.
  Validation: `cargo nextest run -p nako-server playback --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`.
  Evidence: final gate results in `EVIDENCE_AND_GATES.md`.
  Review: follow-ons remain split from this startup boundary.
