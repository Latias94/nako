# Scan Staging Pressure Admission Evidence

Date: 2026-06-03

## Selected Slice

- Implemented the smallest useful follow-on after `04b`: reuse the existing
  library scan admission seam and existing staging manifest pressure accounting.
- Added a shared server-side `StorageStagingPressureStatus` classifier so Admin
  diagnostics and scan admission use the same thresholds.
- Synchronous scan admission only blocks libraries that require remote probe
  staging. Local libraries remain compatible even when remote staging pressure
  is critical.
- Queued background scan scheduling uses global staging pressure as a queue
  protection gate so critical pressure does not claim jobs and immediately
  fail-drain the durable queue.
- No schema migration, Admin DTO, Public Client API change, or operator-facing
  diagnostic expansion was added.

## Verification

- `cargo check -p nako-server --tests` passed.
- `cargo nextest run -p nako-server scan_library_rejects_critical_staging_pressure_before_pipeline scan_library_allows_local_library_during_remote_staging_pressure job_scheduler_keeps_background_scan_jobs_queued_while_staging_pressure_is_critical --no-fail-fast` passed: 3 tests passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed.

## Coverage

- Remote synchronous scan rejects critical staging pressure before WebDAV
  `PROPFIND`, so scan/probe work does not start under pressure.
- Local synchronous scan remains allowed under remote staging pressure because
  it does not require remote probe staging.
- Background scan scheduler leaves a queued job queued while staging pressure is
  critical, then schedules and completes it after the manifest pressure is
  cleared.
- Existing Admin staging diagnostics continue to expose the same DTO shape while
  sharing the classifier used by admission.

## Deferred Follow-Ons

- Per-library or per-backend staging pressure budgets instead of global manifest
  pressure.
- Scheduler fairness that can inspect job library IDs before global pressure
  deferral, allowing local queued scans to proceed under remote staging pressure.
- PostgreSQL runtime parity harness for storage admission behavior.
- Watcher/debounce and broader intake pressure policies.
- Operator action flow for cleaning or expiring staging pressure sources.

## Spec Update Judgment

Updated `.trellis/spec/nako-server/backend/directory-structure.md` with the
library scan staging-pressure admission contract because this task changed a
storage/queue control-plane boundary.
