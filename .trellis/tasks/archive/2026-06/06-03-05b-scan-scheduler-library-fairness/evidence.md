# Scan Scheduler Library Fairness Evidence

Date: 2026-06-03

## Selected Slice

- Implemented the smallest useful fairness follow-on after staging-pressure
  admission: queued library scan scheduling now inspects durable claimable
  candidates in lease order before it claims a job.
- Reused the existing per-library storage admission seam instead of changing
  the per-backend budget model owned by 05a.
- Added a read-only durable-job repository preview seam so the scheduler can
  preserve aged-fairness / priority ordering while deciding whether a candidate
  library is runnable.
- Kept the existing runtime supervisor, `disk.scan` concurrency permit, and
  durable lease claim boundary. No new scheduler table or hidden background path
  was added.

## Shipped Behavior

- `LibraryScanAppService::schedule_queued_library_scans` now loads claimable
  `disk.scan` candidates in durable lease order, checks per-library storage
  admission, and skips blocked candidates without claiming them.
- Once a candidate passes admission, the scheduler claims that exact job by
  `job_id` and launches the existing background scan runtime path.
- A blocked remote scan now remains `queued` while a runnable healthy/local
  queued scan can proceed in the same scheduling pass.
- When all currently claimable queued scans are blocked by storage or staging
  pressure, the scheduler returns `BudgetSaturated` and leaves durable queue
  state unchanged.
- SQLite and PostgreSQL now expose `list_claimable_jobs_for_lease(...)` with
  the same aged-fairness / priority / FIFO ordering as `claim_next_job_lease`.

## Boundaries Preserved

- No schema changes or migrations.
- No Public Client API or Admin API contract changes.
- No per-backend staging budget model changes; queued fairness stays on top of
  the existing admission seam.
- No raw `tokio::spawn` path outside the existing durable job runtime boundary.
- No redaction regression: blocked jobs are no longer failed just to discover
  admission pressure.

## Verification

- `cargo check -p nako-server --tests` passed.
- `cargo check -p nako-db --tests` passed.
- `cargo nextest run -p nako-server background_scan_scheduler_skips_blocked_library_and_schedules_runnable_scan --no-fail-fast`
  passed.
- `cargo nextest run -p nako-server job_scheduler_keeps_remote_scan_jobs_queued_while_staging_pressure_is_critical --no-fail-fast`
  passed.
- `cargo nextest run -p nako-db sqlite_job_retry_contract_priority_policy_orders_fairly_and_recovers --no-fail-fast`
  passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed.

## Coverage

- Blocked remote library scans stay queued instead of being claim-and-fail
  drained.
- Runnable healthy/local queued scans can still start when an earlier blocked
  remote scan exists.
- Remote queued scans still remain queued under critical staging pressure and
  run after pressure clears.
- Durable-job claim preview ordering stays aligned with lease claim ordering.

## Deferred Follow-Ons

- Per-backend or per-library staging budget models remain the responsibility of
  05a.
- Broader fairness across non-scan durable job kinds remains a future
  control-plane lane.
- PostgreSQL runtime harness parity for storage/scan pressure remains a
  separate task.
- Watcher/debounce and other intake scheduling policies remain separate from
  this queued fairness slice.

## Spec Update Judgment

Updated `.trellis/spec/nako-server/backend/directory-structure.md` and
`.trellis/spec/nako-db/backend/database-guidelines.md` because this task added
an executable queue-admission contract spanning server scheduling and durable
job repository ordering.
