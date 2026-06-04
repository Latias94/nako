# Playback Release Hardware Report Evidence

## Goal

Make the playback release gate produce durable, operator-readable hardware
diagnostics evidence by serializing the existing `nako-transcode` FFmpeg
hardware capability report to `target/release-gate/playback-hardware-report.json`.

## Requirements

- Add a small `nako-transcode` example binary that:
  - probes an FFmpeg binary through the existing
    `FfmpegHardwareAccelerationDetector`;
  - serializes `HardwareAccelerationReport` as pretty JSON;
  - accepts `--ffmpeg <path>` with default `ffmpeg`;
  - accepts `--output <path>` to write the JSON report, or prints to stdout when
    omitted;
  - creates the parent directory for `--output`;
  - rejects unknown/missing CLI arguments with a non-zero exit.
- Wire both `scripts/release-gate.sh --mode playback` and
  `scripts/release-gate.ps1 -Mode playback` to run the example after the
  existing hardware nextest gate and write
  `target/release-gate/playback-hardware-report.json`.
- Preserve existing release gate behavior:
  - `ffmpeg -version` and `ffprobe -version` remain presence checks;
  - GPU device access is not required for default playback gate success;
  - probe errors are represented by the existing redacted/degraded report
    behavior rather than raw stderr dumps.
- Update operations/deployment docs to describe the generated report as shipped
  evidence and keep container device pass-through as a follow-on.

## Acceptance Criteria

- [x] `cargo run -p nako-transcode --example hardware-report -- --ffmpeg ffmpeg --output target/release-gate/playback-hardware-report.json`
      writes JSON containing `capabilities`.
- [x] The generated JSON does not include raw local paths or token-like
      diagnostic detail when probe failures occur.
- [x] Bash release gate playback mode includes the hardware report step.
- [x] PowerShell release gate playback mode includes the hardware report step.
- [x] Operations/release docs no longer imply the release gate lacks hardware
      report evidence.
- [x] Focused `nako-transcode` check/tests pass.

## Definition Of Done

- Code and scripts are formatted/lint-safe for the edited surfaces.
- `cargo check -p nako-transcode --examples --tests` passes.
- `cargo nextest run -p nako-transcode hardware --no-fail-fast` passes.
- The hardware report example is exercised locally when FFmpeg is available.
- `git diff --check` and Trellis task validation pass.
- Task evidence records commands and any environment limitation.

## Technical Approach

Use an example instead of adding a production CLI command because this is a
release-gate evidence producer, not a public runtime surface. The example should
reuse the exported detector and report types without adding new dependencies or
duplicating probe parsing.

The script step should be explicit and cross-platform:

- Bash writes to `$release_gate_output/playback-hardware-report.json`.
- PowerShell writes to `$ReleaseGateOutput/playback-hardware-report.json`.

## Decision (ADR-lite)

Context: The hardware diagnostic model already includes capability stages,
device initialization state, smoke probe state, and redacted probe errors, but
the release gate only runs tests and does not persist the host report.

Decision: Generate a JSON hardware report during playback release gates using a
small `nako-transcode` example and the existing detector.

Consequences: Operators and release reviewers get an auditable host capability
matrix without requiring GPU hardware to pass the default gate. Full container
device pass-through smoke remains a separate operational follow-on.

## Out Of Scope

- Requiring VAAPI/NVENC/QSV/AMF/VideoToolbox devices in default release gates.
- Running real one-frame GPU encode probes from the release gate.
- Adding a public server/admin route or changing Admin DTOs.
- Changing hardware selection, fallback, HLS command planning, or playback
  policy.
- Solving Docker device pass-through; this task only records local FFmpeg
  capability evidence.

## Technical Notes

- `scripts/release-gate.sh` and `scripts/release-gate.ps1` already have
  playback mode and hardware nextest checks.
- `crates/nako-transcode/src/hardware.rs` already owns FFmpeg hardware
  detection and redacted probe-failure reports.
- `docs/architecture/OPERATIONS_RELEASE.md` currently lists hardware readiness
  diagnostics as partial and the next lane as per-host FFmpeg/hardware smoke
  matrix.
