# Library Scan Scheduling / Storage Admission Evidence

Date: 2026-06-03
Selected slice: library scan storage-health admission at scan job execution entry.

## Selection

Chose the bounded library scan admission slice because Nako already has:

- durable `Storage Backend Health` rows;
- `Storage Circuit Breaker` persistence and reset;
- process-local and durable VFS backoff rejection inside `LibraryStorageBackend`;
- durable background library scan jobs with typed `disk.scan` budget admission.

The smallest useful product-readiness step was to make library scan execution
respect the existing storage circuit state before entering scan, probe, or
metadata acquisition work. This closes a real control-plane gap without
rewriting scan orchestration, adding a new scheduler service, or changing
schema.

## Shipped Behavior

- `LibraryScanAppService::run_library_scan` now performs a storage admission
  check before index/probe/metadata work starts.
- The admission check reuses the library backend durable health lookup and the
  same redaction-safe `storage circuit breaker is open` error shape already
  used by VFS operations.
- Synchronous `scan_library` and queued background scan jobs now share the same
  storage-health admission behavior.
- When a queued scan is rejected by storage admission, the failed job releases
  `disk.scan` budget and the scheduler follow-up can continue to the next
  queued scan.
- No local path, Source Locator, Source Fingerprint, backend URL, signed URL,
  credential, or raw backend error is exposed by the new failure path.

## Boundaries Preserved

- No schema changes or migration updates.
- No Public Client API or Admin API contract changes.
- No VFS local/WebDAV adapter behavior change beyond reusing existing durable
  circuit semantics for library scan admission.
- No raw `tokio::spawn` scan helper was added outside the existing durable job
  / runtime supervisor path.
- No new storage diagnostic DTO or storage reset route shape was introduced in
  this slice.

## Validation

- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-02-04b-library-scan-scheduling-storage-admission`
  passed.
- `cargo check -p nako-server --tests` passed.
- `cargo nextest run -p nako-server scan_library_rejects_open_storage_circuit_before_pipeline background_scan_job_failure_releases_budget_and_schedules_next_queued_scan --no-fail-fast`
  passed: 2 tests.
- `cargo nextest run -p nako-server background_scan_job_uses_runtime_job_supervision job_scheduler_leaves_background_scan_jobs_queued_until_scan_budget_is_available background_scan_job_acknowledges_cancellation_before_probe_stage --no-fail-fast`
  passed: 3 tests.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with LF/CRLF normalization warnings only.

## Follow-ons

- Source Fingerprint escalation remains separate from scan scheduling admission.
- File watcher / debounce productization remains separate from durable scan
  admission.
- PostgreSQL runtime parity harness for storage-health and scan scheduling
  remains a separate evidence lane.
- Broader scheduler priority / fairness policy across more job kinds remains a
  control-plane follow-on, not part of this slice.
- Staging-pressure-based admission is still separate from durable storage
  circuit admission and can build on the same typed entry boundary later.
