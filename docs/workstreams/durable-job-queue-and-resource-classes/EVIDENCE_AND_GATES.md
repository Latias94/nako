# Durable Job Queue And Resource Classes - Evidence And Gates

Status: Closed
Last updated: 2026-05-29

## Smallest Current Repro

```powershell
cargo nextest run -p nako-server runtime_resource_class --no-fail-fast
```

This proves the first slice: process-local runtime resource classes are
registered once, expose safe diagnostics, and own the existing scan, metadata,
and webhook permit pools.

## Gate Set

### Resource Registry Gate

```powershell
cargo nextest run -p nako-server runtime_resource_class --no-fail-fast
```

Expected coverage:

- duplicate resource class names are rejected;
- diagnostics are sorted and redacted;
- acquiring a class permit changes available permit diagnostics;
- missing resource class lookup fails explicitly.

### Scheduler Mapping Gate

```powershell
cargo nextest run -p nako-server job_resource_class --no-fail-fast
```

Expected coverage:

- durable job classes map to budget classes explicitly;
- unknown mappings do not bypass admission accidentally;
- scan, metadata, NFO, artwork, addon, and webhook classes have covered
  mappings before scheduler migration.

### Scheduler Runtime Gate

```powershell
cargo nextest run -p nako-server job_scheduler --no-fail-fast
cargo nextest run -p nako-server job_runtime --no-fail-fast
```

Expected coverage:

- queue admission respects resource budget saturation;
- leases and cancellation checkpoints still fence terminal writes;
- typed executors remain responsible for domain side effects.

### Cross-Crate Contract Gate

```powershell
cargo nextest run -p nako-db job_retry --no-fail-fast
cargo nextest run -p nako-server queue_pressure --no-fail-fast
cargo check -p nako-core -p nako-db -p nako-api -p nako-server --tests
```

Expected coverage:

- any persisted retry/backoff or queue pressure records compile across the
  repository, DB, API, and server boundaries.
- retry attempts create new durable job rows instead of mutating failed jobs
  back to queued;
- queue pressure diagnostics remain redacted and grouped by safe job fields.

### Formatting And Static Gate

```powershell
cargo fmt --all -- --check
git diff --check
Get-Content docs\workstreams\durable-job-queue-and-resource-classes\WORKSTREAM.json | ConvertFrom-Json
```

Expected coverage:

- Rust formatting is stable;
- patches have no whitespace errors;
- workstream metadata remains parseable JSON.

## Evidence Anchors

- `docs/workstreams/durable-job-queue-and-resource-classes/DESIGN.md`
- `docs/workstreams/durable-job-queue-and-resource-classes/TODO.md`
- `crates/nako-server/src/app/runtime.rs`
- `crates/nako-server/src/app/composition.rs`
- `crates/nako-core/src/job.rs`
- `crates/nako-core/src/repository/jobs.rs`
- `crates/nako-db/src/sqlite/jobs.rs`
- `crates/nako-db/src/postgres/jobs.rs`
- `crates/nako-server/src/app/job_runtime.rs`
- `crates/nako-server/src/app/jobs.rs`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/architecture/WORKSTREAM_LINKS.md`

## Current Evidence

- `DJRC-010` (2026-05-29): Lane opened from the Control Plane proposed
  candidate. Scope freezes a `nako-server`-local first slice, no new crate, no
  external queue, no public wire DTO, and no broad worker migration.
- `DJRC-020` (2026-05-29): Added `RuntimeResourceClassRegistry` in
  `nako-server::app::runtime`, centralized process-local resource classes for
  `disk.scan`, `metadata.shared`, and `network.webhook`, and made
  `NakoRuntimeResources` source existing scan, metadata, and webhook
  semaphores from the registry. Added focused tests for duplicate rejection,
  unknown class rejection, sorted diagnostics, permit accounting, and app
  composition diagnostics. Gates run:
  `cargo nextest run -j 2 -p nako-server runtime_resource_class --no-fail-fast`
  (4 passed, 454 skipped); `cargo fmt --all -- --check` (pass);
  `git diff --check` (pass, CRLF warnings only);
  `Get-Content docs\workstreams\durable-job-queue-and-resource-classes\WORKSTREAM.json | ConvertFrom-Json`
  (pass). An initial empty-library version of the app composition test failed
  with `server config must include at least one library`; the test fixture was
  corrected to include a minimal local library before the passing gate run.
- `DJRC-030` (2026-05-29): Added explicit durable job kind/resource class to
  runtime budget class mapping. Immediate durable `spawn_job` callers for
  library scan, metadata refresh/maintenance, NFO import/export, and direct
  addon task dispatch now pass mapped budget classes to `RuntimeSupervisor`.
  New addon task jobs use fixed `addon.task` instead of declaration-derived
  resource classes; legacy `addon.task.*` and
  `addon.generated_artifact_handoff` rows remain mappable. Added default
  registry classes for `artwork.ingest` and `addon.task`. Gates run:
  `cargo nextest run -j 2 -p nako-server job_resource_class --no-fail-fast`
  (2 passed, 458 skipped); `cargo check -j 2 -p nako-server --tests` (pass);
  `cargo nextest run -j 2 -p nako-server runtime_resource_class --no-fail-fast`
  (4 passed, 456 skipped); `cargo fmt --all -- --check` (pass);
  `git diff --check` (pass, CRLF warnings only).
- `DJRC-040` (2026-05-29): Added the first typed scheduler admission tracer
  bullet for library scan jobs. `DurableJobRuntime` can now run an already
  claimed `LeasedJob`, allowing the scheduler to claim by kind/resource class
  only after a scan budget permit is available. `enqueue_library_scan` now asks
  the scheduler to admit queued scan work instead of spawning an unbounded task
  per enqueue; completed scan jobs trigger a short supervised scheduler
  follow-up. The new `job_scheduler` test proves the second background scan job
  remains `queued` while the first job holds the scan budget, then succeeds
  after the first job releases the budget. Gates run:
  `cargo nextest run -j 2 -p nako-server job_scheduler --no-fail-fast`
  (1 passed, 460 skipped); `cargo nextest run -j 2 -p nako-server job_runtime --no-fail-fast`
  (5 passed, 456 skipped); `cargo nextest run -j 2 -p nako-server background_scan_job --no-fail-fast`
  (3 passed, 458 skipped); `cargo check -j 2 -p nako-server --tests` (pass).
- `DJRC-050` (2026-05-29): Added persisted retry/backoff metadata to durable
  jobs in SQLite and PostgreSQL baselines; `enqueue_job_retry` creates a new
  queued job row that copies the failed source job input, increments the
  attempt, and records `retry_of_job_id` plus optional `next_attempt_at`.
  Lease claims now skip queued jobs whose retry backoff is still in the future,
  and claiming/running work clears due backoff state. Added redacted queue
  pressure summaries grouped by job kind, status, and resource class with
  counts, claimable count, delayed retry count, oldest queued time, and next
  retry time. Cancellation is rejected as a retry source. A SQLite managed
  artwork private job query was updated to select the new job retry fields.
  Gates run: `cargo nextest run -j 2 -p nako-db job_retry --no-fail-fast`
  (1 passed, 156 skipped); `cargo nextest run -j 2 -p nako-server queue_pressure --no-fail-fast`
  (1 passed, 461 skipped); `cargo nextest run -j 2 -p nako-db job_lease --no-fail-fast`
  (4 passed, 153 skipped); `cargo nextest run -j 2 -p nako-server job_scheduler --no-fail-fast`
  (1 passed, 461 skipped); `cargo nextest run -j 2 -p nako-server job_runtime --no-fail-fast`
  (5 passed, 457 skipped); `cargo nextest run -j 2 -p nako-db managed_artwork --no-fail-fast`
  (12 passed, 145 skipped); `cargo check -j 2 -p nako-core -p nako-db -p nako-api -p nako-server --tests`
  (pass); `cargo fmt --all -- --check` (pass); `git diff --check` (pass, CRLF
  warnings only). PostgreSQL contract variants remain `#[ignore]` in the local
  test harness unless `NAKO_TEST_POSTGRES_URL` is provided; this run validated
  PostgreSQL compile coverage and SQLite runtime behavior.
- `DJRC-060` (2026-05-29): Closeout review found no blocking findings for
  workstream compliance or code quality. The lane is closed with shipped
  process-local resource classes, explicit durable job to budget mapping,
  library scan scheduler admission, persisted retry/backoff, and redacted queue
  pressure diagnostics. Residual priority policy, distributed scheduling,
  remote workers, addon process lifecycle, child-process cancellation, and
  broader job-kind scheduler migration are explicitly split as follow-ons.
  Fresh gates run:
  `cargo nextest run -j 2 -p nako-server runtime_resource_class --no-fail-fast`
  (4 passed, 458 skipped);
  `cargo nextest run -j 2 -p nako-server job_resource_class --no-fail-fast`
  (2 passed, 460 skipped);
  `cargo nextest run -j 2 -p nako-server job_scheduler --no-fail-fast`
  (1 passed, 461 skipped);
  `cargo nextest run -j 2 -p nako-server job_runtime --no-fail-fast`
  (5 passed, 457 skipped);
  `cargo nextest run -j 2 -p nako-db job_retry --no-fail-fast`
  (1 passed, 156 skipped);
  `cargo nextest run -j 2 -p nako-server queue_pressure --no-fail-fast`
  (1 passed, 461 skipped);
  `cargo check -j 2 -p nako-core -p nako-db -p nako-api -p nako-server --tests`
  (pass);
  `Get-Content docs\workstreams\durable-job-queue-and-resource-classes\WORKSTREAM.json | ConvertFrom-Json`
  (pass);
  `cargo fmt --all -- --check` (pass);
  `git diff --check` (pass, CRLF warnings only).

## Notes

Fresh verification is required before marking a task, Codex goal, or lane
complete. Resource diagnostics must stay budget-oriented and must not expose
raw job payloads, provider responses, filesystem paths, storage handles, source
locators, tokens, or secrets.
