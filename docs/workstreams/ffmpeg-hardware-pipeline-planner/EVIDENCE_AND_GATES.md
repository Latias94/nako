# FFmpeg Hardware Pipeline Planner - Evidence And Gates

Status: Complete
Last updated: 2026-05-27

## Gate Commands

- `python -m json.tool docs/workstreams/ffmpeg-hardware-pipeline-planner/WORKSTREAM.json`
- `cargo nextest run -p nako-transcode --no-fail-fast`
- `cargo nextest run -p nako-playback --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo nextest run -p nako-api --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## Evidence Log

### FHPP-010

Status: Complete
Date: 2026-05-27

Evidence:

- Added ADR 0045.
- Created workstream docs and task ledger.

Validation:

- `python -m json.tool docs/workstreams/ffmpeg-hardware-pipeline-planner/WORKSTREAM.json > $null` passed.

### FHPP-020

Status: Complete
Date: 2026-05-27

Evidence:

- Added stage-aware hardware capability records for decode, filter, encode,
  hwaccel, and bitstream-filter facts.
- Expanded hardware capability modeling to include CPU, NVENC, VAAPI, QSV, AMF,
  and VideoToolbox.
- Preserved device initialization and smoke-probe evidence as redaction-safe
  runtime facts.

Validation:

- `cargo nextest run -p nako-transcode --no-fail-fast` passed, 41 tests.

### FHPP-030

Status: Complete
Date: 2026-05-27

Evidence:

- Added `TranscodePipelinePlanner`, `TranscodePipelineRequest`, and
  `TranscodePipelinePlan`.
- Removed the old selected-accelerator helper path from production code.
- Planner now emits fallback evidence and typed readiness reasons for CPU
  fallback, fail policy, and probe-error degradation.

Validation:

- `cargo nextest run -p nako-transcode --no-fail-fast` passed, 41 tests.

### FHPP-040

Status: Complete
Date: 2026-05-27

Evidence:

- HLS playback now receives a pipeline-derived execution policy from
  `HlsAppService`.
- `FfmpegCommandBuilder` consumes planned decode/filter/encode stages instead
  of selecting hardware from config or probe output.
- VAAPI, QSV, AMF, VideoToolbox, NVENC, and CPU command paths are represented
  behind the transcode policy boundary.

Validation:

- `cargo nextest run -p nako-transcode --no-fail-fast` passed, 41 tests.
- `cargo nextest run -p nako-playback --no-fail-fast` passed, 17 tests.
- `cargo nextest run -p nako-server playback --no-fail-fast` passed, 81 tests.

### FHPP-050

Status: Complete
Date: 2026-05-27

Evidence:

- Admin playback diagnostics now expose `pipeline` readiness instead of the old
  `selection` field.
- Admin hardware capability DTOs include stage capability facts.
- Public Client API remains hardware-redacted; SDK/admin contract tests pass.

Validation:

- `cargo nextest run -p nako-api --no-fail-fast` passed, 61 tests.
- `cargo nextest run -p nako-server playback --no-fail-fast` passed, 81 tests.

### FHPP-060

Status: Complete
Date: 2026-05-27

Evidence:

- Workstream TODO, milestones, evidence, handoff, and state metadata were
  refreshed after implementation.
- Follow-ons are documented in the handoff instead of hidden as incomplete
  tasks in this lane.

Validation:

- `python -m json.tool docs/workstreams/ffmpeg-hardware-pipeline-planner/WORKSTREAM.json > $null` passed.
- `cargo nextest run -p nako-transcode --no-fail-fast` passed, 41 tests.
- `cargo nextest run -p nako-playback --no-fail-fast` passed, 17 tests.
- `cargo nextest run -p nako-api --no-fail-fast` passed, 61 tests.
- `cargo nextest run -p nako-server playback --no-fail-fast` passed, 81 tests.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with only Git CRLF conversion warnings.
- Broader workspace gates were not run because this lane touched the
  playback/transcode/admin diagnostics boundary; the focused package gates cover
  that behavioral surface.

## Review Checks

- No copied Jellyfin code, comments, tests, schemas, or assets.
- No Public Client leakage of FFmpeg command strings, local paths, or hardware
  device details.
- FFmpeg command builder does not own hardware policy selection.
- Server playback app does not branch on FFmpeg encoder names.
- Hardware diagnostics distinguish decode, filter, encode, hwaccel, device, and
  smoke-probe facts.
