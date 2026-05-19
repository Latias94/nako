# NFO Sidecar Cancellation Checkpoints - Milestones

Status: Complete
Last updated: 2026-05-19

## M0 - Scope And Boundary Freeze

Exit criteria:

- The app-level NFO cancellation gap is stated.
- `taru-nfo` owns the service-level checkpoint contract.
- Retry/backoff, lease stealing, and child-process cancellation are explicit
  non-goals.

Primary evidence:

- `docs/workstreams/nfo-sidecar-cancellation-checkpoints/DESIGN.md`
- `docs/workstreams/nfo-sidecar-cancellation-checkpoints/TODO.md`

## M1 - Service Contract

Exit criteria:

- `taru-nfo` has redacted checkpoint payload types.
- Library import/export have checkpoint-aware variants.
- Existing callers can keep using no-op import/export APIs.
- Cancellation is distinct from NFO failure.

Primary gates:

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
$env:CARGO_BUILD_JOBS='2'
$env:NEXTEST_TEST_THREADS='1'
cargo check -j 2 -p taru-nfo --tests
cargo nextest run -j 2 -p taru-nfo nfo_service --no-fail-fast
```

## M2 - Import Checkpoints

Exit criteria:

- Import checks before every source sidecar unit.
- A cancelled checkpoint stops the next import sidecar.
- Partial summary is safe and does not treat cancellation as failure.

Primary gate:

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
$env:CARGO_BUILD_JOBS='2'
$env:NEXTEST_TEST_THREADS='1'
cargo nextest run -j 2 -p taru-nfo import --no-fail-fast
```

## M3 - Export Checkpoints

Exit criteria:

- Export checks before every source sidecar unit.
- A cancelled checkpoint prevents the next sidecar write/backup.
- `export_media_source` behavior remains compatible unless intentionally
  extended with tests.

Primary gate:

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
$env:CARGO_BUILD_JOBS='2'
$env:NEXTEST_TEST_THREADS='1'
cargo nextest run -j 2 -p taru-nfo export --no-fail-fast
```

## M4 - Server Integration

Exit criteria:

- Server maps durable job cancellation into NFO per-sidecar checkpoints.
- Cancelled NFO import/export jobs persist terminal `cancelled`.
- `NfoImported` and `NfoExported` events are not emitted after cancellation.
- Admin docs remain redacted and boundary-based.

Primary gates:

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
$env:CARGO_BUILD_JOBS='2'
$env:NEXTEST_TEST_THREADS='1'
cargo nextest run -j 2 -p taru-server nfo --no-fail-fast
cargo nextest run -j 2 -p taru-server job_cancel --no-fail-fast
```

## M5 - Closeout

Exit criteria:

- Gate set is recorded with fresh evidence.
- Workstream status matches reality.
- Remaining NFO cancellation edge cases are completed or split.

Result:

- Complete. No follow-on split was required for this lane's NFO-specific
  cancellation scope.
