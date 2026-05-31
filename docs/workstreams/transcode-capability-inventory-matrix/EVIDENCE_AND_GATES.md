# Transcode Capability Inventory Matrix - Evidence And Gates

Status: Closed
Last updated: 2026-05-31

## Required Gates

```text
python -m json.tool docs/workstreams/transcode-capability-inventory-matrix/WORKSTREAM.json
cargo nextest run -p nako-transcode hardware --no-fail-fast
cargo nextest run -p nako-transcode probe --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Run focused transcode capability gates only. Broaden to HLS/server gates only
if the worker reports an approved scope expansion, which should normally block
this workstream and return to planner coordination.

## Evidence Ledger

### TCIM-010 - Scope and evidence freeze

Status: Done

Evidence:

- `DESIGN.md`
- `TODO.md`
- `WORKSTREAM.json`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/LANES.md`
- `docs/workstreams/README.md`

Notes:

- This lane is safe to run beside HDR `HTP-030` only while it stays in
  `hardware.rs` / `probe.rs` inventory and report seams.
- Pipeline selection, FFmpeg command planning, server routes, API DTOs, and
  release packaging are explicit follow-ons.

### TCIM-020 - Bitstream filter inventory baseline

Status: Done

Evidence:

- `crates/nako-transcode/src/hardware.rs`
- `crates/nako-transcode/src/lib.rs`
- `hardware_static_report_expresses_bitstream_filter_capability_as_optional`
- `hardware_probe_report_keeps_missing_bitstream_filter_optional_for_selection`

Findings:

- Added optional static stage evidence for the `h264_mp4toannexb` bitstream
  filter.
- CPU and static hardware detector capabilities now advertise the optional
  bitstream-filter stage.
- Probe-derived reports can represent a missing optional bitstream filter
  without treating the accelerator as unusable.
- HLS pipeline selection does not change when optional bitstream-filter
  evidence is missing.
- Broader decoder, encoder, filter, tone-map, and subtitle coverage is split to
  `TCIM-030`.

Verification on 2026-05-31:

- `cargo nextest run -p nako-transcode hardware --no-fail-fast` passed with
  11 tests run.
- `cargo nextest run -p nako-transcode probe --no-fail-fast` passed with 10
  tests run.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with only Windows line-ending normalization
  warnings.

### TCIM-030 - Broader inventory matrix facts

Status: Done

Evidence:

- `crates/nako-transcode/src/hardware.rs`
- `crates/nako-transcode/src/lib.rs`
- `hardware_probe_report_exposes_broader_inventory_facts_without_selecting_them`
- `hardware_probe_report_keeps_missing_broader_inventory_facts_optional`

Findings:

- Added report-stage names for tone-map and subtitle burn-in evidence.
- Probe-derived reports now include optional broader decoder facts for HEVC and
  AV1 plus hardware-specific decoder facts for CUDA/NVDEC and QSV names.
- Probe-derived reports now include optional future encoder facts for CPU,
  VAAPI, NVENC, QSV, AMF, and VideoToolbox names.
- Probe-derived reports now include optional filter facts for common software
  filters and selected VAAPI, CUDA, and QSV filter names.
- Probe-derived reports now include optional tone-map filter evidence and
  subtitle burn-in filter evidence.
- Listed broader facts are observable in `HardwareAccelerationReport`; missing
  broader facts are represented as `Missing` with `required=false`.
- HLS pipeline selection does not change when broader facts are listed or
  missing.

Verification on 2026-05-31:

- `cargo nextest run -p nako-transcode hardware --no-fail-fast` passed with
  13 tests run.
- `cargo nextest run -p nako-transcode probe --no-fail-fast` passed with 12
  tests run.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with only Windows line-ending normalization
  warnings.

Planner acceptance on 2026-05-31:

- `TCIM-030` review found no blocking workstream or code-quality findings.
- Fresh planner verification reran the required hardware, probe, formatting,
  manifest, and whitespace gates.
- Commit `bb53bd98 feat(transcode): expand capability inventory facts` was
  merged into `main`.

### TCIM-040 - Verification and closeout

Status: Done

Evidence:

- `docs/workstreams/transcode-capability-inventory-matrix/CLOSEOUT.md`
- `docs/workstreams/transcode-capability-inventory-matrix/WORKSTREAM.json`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/LANES.md`
- `docs/workstreams/README.md`

Findings:

- The lane shipped optional capability inventory facts without changing
  runtime selection or FFmpeg command planning.
- Hardware tone-map execution, HEVC/AV1 output policy, subtitle burn-in,
  Public/Admin DTO reporting, release hardware matrices, and HLS lifecycle or
  resource admission changes are follow-ons.

## Residual Risks

- The lane improves capability observability, not actual playback format
  breadth.
- Host and Docker driver smoke evidence remains a release/operations follow-on.
