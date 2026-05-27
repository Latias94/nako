# FFmpeg Hardware Pipeline Planner - TODO

Status: Complete
Last updated: 2026-05-27

## M0 - Workstream Open

- [x] FHPP-010 [owner=codex] [deps=none] [scope=docs/workstreams/ffmpeg-hardware-pipeline-planner,docs/adr]
  Goal: Open the lane and record the ADR for pipeline planning ownership.
  Validation: `python -m json.tool docs/workstreams/ffmpeg-hardware-pipeline-planner/WORKSTREAM.json`;
  `git diff --check -- docs/workstreams/ffmpeg-hardware-pipeline-planner docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`.
  Evidence: ADR 0045 and initial workstream docs.
  Handoff: FHPP-020 is the first implementation slice.

## M1 - Capability Inventory

- [x] FHPP-020 [owner=codex] [deps=FHPP-010] [scope=crates/nako-transcode/src/hardware.rs,crates/nako-transcode/src/lib.rs]
  Goal: Replace encoder-only capability reporting with stage-aware hardware
  inventory records for decoder, encoder, filter, hwaccel, bitstream-filter,
  device initialization, and smoke probe evidence.
  Validation: `cargo nextest run -p nako-transcode hardware --no-fail-fast`.
  Review: public transcode crate exports should describe runtime capability
  facts, not FFmpeg command strings.
  Evidence: `cargo nextest run -p nako-transcode --no-fail-fast`.
  Handoff: FHPP-030 consumes these facts in a planner.

## M2 - Pipeline Planner

- [x] FHPP-030 [owner=codex] [deps=FHPP-020] [scope=crates/nako-transcode/src/pipeline.rs,crates/nako-transcode/src/policy.rs,crates/nako-transcode/src/profile.rs]
  Goal: Add `TranscodePipelinePlanner`, request/plan/result records, fallback
  evidence, and profile identity integration; delete shallow selection helpers
  where no longer needed.
  Validation: `cargo nextest run -p nako-transcode pipeline --no-fail-fast`.
  Review: planner tests must cover CPU, NVENC encode-only, VAAPI full pipeline,
  QSV, unavailable fallback-to-CPU, and fail-policy rejection.
  Evidence: `cargo nextest run -p nako-transcode --no-fail-fast`.
  Handoff: FHPP-040 adapts FFmpeg command planning.

## M3 - FFmpeg Adapter

- [x] FHPP-040 [owner=codex] [deps=FHPP-030] [scope=crates/nako-transcode/src/ffmpeg.rs,crates/nako-server/src/app/playback/hls.rs,crates/nako-server/src/app/playback/mod.rs]
  Goal: Make HLS command planning consume the pipeline-derived execution policy
  and remove direct hardware policy decisions from the command builder.
  Validation: `cargo nextest run -p nako-transcode --no-fail-fast`;
  `cargo nextest run -p nako-server playback --no-fail-fast`.
  Review: server app code should not branch on FFmpeg encoder names.
  Evidence: `cargo nextest run -p nako-transcode --no-fail-fast`;
  `cargo nextest run -p nako-server playback --no-fail-fast`.
  Handoff: FHPP-050 updates diagnostics and generated contracts if needed.

## M4 - Diagnostics And Contracts

- [x] FHPP-050 [owner=codex] [deps=FHPP-040] [scope=crates/nako-api/src,crates/nako-server/src/http/admin.rs,sdk/typescript,sdk/kotlin]
  Goal: Update Admin playback runtime/support diagnostics to expose
  stage-aware readiness while preserving redaction; regenerate SDKs if contract
  shape changes.
  Validation: `cargo nextest run -p nako-api --no-fail-fast`;
  `cargo nextest run -p nako-server -E 'test(admin_v1_playback_runtime) | test(playback_support)' --no-fail-fast`.
  Review: Public Client API should not gain hardware pipeline fields.
  Evidence: `cargo nextest run -p nako-api --no-fail-fast`;
  `cargo nextest run -p nako-server playback --no-fail-fast`. No generated SDK
  refresh was required because the changed playback diagnostics are Admin-only
  and public SDK/admin contract gates pass.
  Handoff: FHPP-060 closes the lane.

## M5 - Closeout

- [x] FHPP-060 [owner=codex] [deps=FHPP-050] [scope=docs/workstreams/ffmpeg-hardware-pipeline-planner]
  Goal: Refresh evidence, run final gates, split follow-ons, and mark the lane
  complete.
  Validation: `cargo nextest run -p nako-transcode --no-fail-fast`;
  `cargo nextest run -p nako-server playback --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`.
  Review: remaining work must be explicit follow-ons, not hidden TODOs.
  Evidence: final closeout gates passed on 2026-05-27.
