# Lane D Playback Smoke Evidence

Date: 2026-06-05
Lane: Hardware / Release Smoke Evidence
Status: DONE

## Scope

Implemented non-invasive self-hosted playback smoke evidence:

- playback release gate now runs `cargo nextest run -p nako-transcode hardware --no-fail-fast`;
- hardware diagnostics redact probe failure details before serializing capability reports;
- hardware tests cover CPU HLS readiness and JSON serialization;
- server self-host smoke checks Admin playback support evidence and redaction flags.

No GPU device is required. Hardware accelerator smoke probes still report
`not_run` unless an explicit detector supplies host-specific evidence.

## Changed Files

- `Cargo.lock`
- `crates/nako-transcode/Cargo.toml`
- `crates/nako-transcode/src/hardware.rs`
- `crates/nako-transcode/src/lib.rs`
- `crates/nako-server/src/http/tests/self_host_smoke.rs`
- `scripts/release-gate.sh`
- `scripts/release-gate.ps1`
- `docs/deployment/RELEASE_CHECKLIST.md`

## Validation

Passed:

- `cargo fmt --all`
- `cargo nextest run -p nako-transcode hardware --no-fail-fast`
  - 16 tests passed, 106 skipped.
- `cargo nextest run -p nako-server self_host_smoke --no-fail-fast`
  - 1 test passed, 593 skipped.
- `cargo check -p nako-transcode --tests`

Final integration:

- `cargo check -p nako-playback -p nako-transcode -p nako-server --tests`
  passed after the parallel playback runtime-session lane was integrated.

## Notes

- The hardware report redaction intentionally replaces sensitive probe-detail
  tokens with `<redacted>` while preserving safe operator categories such as
  `filters denied`.
- The release gate command shape remains `--mode playback`; the gate contents
  now include explicit hardware diagnostics evidence.
- No playback session runtime modules, HEVC/AV1 executable HLS output,
  schema/API contracts, generated contracts, or GPU-required CI behavior were
  changed by this lane.
