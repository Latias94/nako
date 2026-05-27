# FFmpeg Probe Inventory - TODO

Status: Complete
Last updated: 2026-05-27

## M0 - Workstream Open

- [x] FPI-010 [owner=codex] [deps=none] [scope=docs/workstreams/ffmpeg-probe-inventory,docs/adr]
  Goal: Open the lane and record ADR 0046 for structured FFmpeg probe
  inventory ownership.
  Validation: `python -m json.tool docs/workstreams/ffmpeg-probe-inventory/WORKSTREAM.json`.
  Evidence: ADR 0046 and initial workstream docs.
  Handoff: FPI-020 implements parser and inventory records.

## M1 - Probe Inventory Parser

- [x] FPI-020 [owner=codex] [deps=FPI-010] [scope=crates/nako-transcode/src/hardware.rs,crates/nako-transcode/src/lib.rs]
  Goal: Add `FfmpegProbeInventory`, parsers for encoders, decoders, hwaccels,
  filters, and bitstream filters, plus tests for representative FFmpeg list
  output.
  Validation: `cargo nextest run -p nako-transcode ffmpeg_probe --no-fail-fast`.
  Review: parser output must be redaction-safe names only, not raw command
  output.
  Evidence: `cargo nextest run -p nako-transcode --no-fail-fast`.
  Handoff: FPI-030 maps inventory into hardware stage capabilities.

## M2 - Stage Capability Mapping

- [x] FPI-030 [owner=codex] [deps=FPI-020] [scope=crates/nako-transcode/src/hardware.rs,crates/nako-transcode/src/pipeline.rs]
  Goal: Build `HardwareAccelerationReport` from inventory facts so decode,
  hwaccel, filter, encode, and bitstream-filter stages are listed/missing from
  real probe data.
  Validation: `cargo nextest run -p nako-transcode hardware --no-fail-fast`.
  Review: pipeline readiness should still be conservative and current HLS
  behavior should not silently enable unsupported hardware.
  Evidence: `cargo nextest run -p nako-transcode --no-fail-fast`.
  Handoff: FPI-040 wires detector startup execution.

## M3 - Detector Execution

- [x] FPI-040 [owner=codex] [deps=FPI-030] [scope=crates/nako-transcode/src/hardware.rs,crates/nako-server/src/app/playback]
  Goal: Make `FfmpegHardwareAccelerationDetector` run bounded FFmpeg discovery
  commands for each inventory list and degrade to probe-error reports when a
  required command fails.
  Validation: `cargo nextest run -p nako-transcode --no-fail-fast`;
  `cargo nextest run -p nako-server playback --no-fail-fast`.
  Review: command execution must not expose raw paths or full command output in
  diagnostics.
  Evidence: `cargo nextest run -p nako-transcode --no-fail-fast`;
  `cargo nextest run -p nako-server playback --no-fail-fast`.
  Handoff: FPI-050 updates Admin diagnostics/tests.

## M4 - Diagnostics And Closeout

- [x] FPI-050 [owner=codex] [deps=FPI-040] [scope=crates/nako-api/src/admin/playback.rs,crates/nako-server/src/http/admin.rs,docs/workstreams/ffmpeg-probe-inventory]
  Goal: Update Admin tests/evidence for richer stage capabilities, run final
  gates, document follow-ons, and close the lane.
  Validation: `cargo nextest run -p nako-api --no-fail-fast`;
  `cargo nextest run -p nako-server playback --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`.
  Review: Public Client API should remain hardware-redacted.
  Evidence: final closeout gates passed on 2026-05-27.
