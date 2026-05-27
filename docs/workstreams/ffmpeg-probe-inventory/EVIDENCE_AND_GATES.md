# FFmpeg Probe Inventory - Evidence And Gates

Status: Complete
Last updated: 2026-05-27

## Gate Commands

- `python -m json.tool docs/workstreams/ffmpeg-probe-inventory/WORKSTREAM.json`
- `cargo nextest run -p nako-transcode --no-fail-fast`
- `cargo nextest run -p nako-api --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## Evidence Log

### FPI-010

Status: Complete
Date: 2026-05-27

Evidence:

- Added ADR 0046.
- Created workstream docs and task ledger.

Validation:

- `python -m json.tool docs/workstreams/ffmpeg-probe-inventory/WORKSTREAM.json > $null` passed.

### FPI-020

Status: Complete
Date: 2026-05-27

Evidence:

- Added `FfmpegProbeInventory` with redaction-safe parsed name sets for
  encoders, decoders, hwaccels, filters, and bitstream filters.
- Added parser coverage for FFmpeg headers, explanatory lines, and capability
  rows.

Validation:

- `cargo nextest run -p nako-transcode --no-fail-fast` passed, 44 tests.

### FPI-030

Status: Complete
Date: 2026-05-27

Evidence:

- `HardwareAccelerationReport` now maps stage capabilities from probe inventory
  facts.
- Stage capabilities now record whether a capability is required or optional.
- Current HLS hardware availability remains conservative: required
  decoder/hwaccel/filter/encoder stages must be available.

Validation:

- `cargo nextest run -p nako-transcode --no-fail-fast` passed, 44 tests.

### FPI-040

Status: Complete
Date: 2026-05-27

Evidence:

- `FfmpegHardwareAccelerationDetector` now runs `-encoders`, `-decoders`,
  `-hwaccels`, `-filters`, and `-bsfs`.
- Detector tests cover successful multi-command probing and probe-error
  degradation.
- HLS hardware decode arguments are emitted before `-i`, matching FFmpeg input
  option ordering.
- Server fake FFmpeg scripts were updated so startup probing does not interfere
  with playback runner behavior.

Validation:

- `cargo nextest run -p nako-transcode --no-fail-fast` passed, 44 tests.
- `cargo nextest run -p nako-server playback --no-fail-fast` passed, 81 tests.

### FPI-050

Status: Complete
Date: 2026-05-27

Evidence:

- Admin stage capability diagnostics include `required` so optional
  bitstream-filter evidence can be visible without blocking current HLS
  availability.
- Public Client API remains hardware-redacted.
- Workstream docs and state metadata were refreshed after implementation.

Validation:

- `python -m json.tool docs/workstreams/ffmpeg-probe-inventory/WORKSTREAM.json > $null` passed.
- `cargo nextest run -p nako-transcode --no-fail-fast` passed, 44 tests.
- `cargo nextest run -p nako-api --no-fail-fast` passed, 61 tests.
- `cargo nextest run -p nako-server playback --no-fail-fast` passed, 81 tests.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with only Git CRLF conversion warnings.
- Broader workspace gates were not run because this lane touched the
  transcode/probe/Admin playback boundary; the focused package gates cover that
  behavioral surface.

## Review Checks

- No copied Jellyfin code, comments, tests, schemas, or assets.
- Probe output is reduced to redaction-safe capability names.
- FFmpeg command builders do not parse probe output.
- Pipeline planner continues to consume `HardwareAccelerationReport`.
- Admin diagnostics expose stage evidence without public API leakage.
