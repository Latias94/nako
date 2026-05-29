# Durable Job Queue And Resource Classes - TODO

Status: Closed
Last updated: 2026-05-29

## M0 - Scope And Evidence Freeze

- [x] DJRC-010 [owner=planner] [deps=none] [scope=docs/workstreams/durable-job-queue-and-resource-classes,docs/architecture]
  Goal: Open the lane, freeze target/non-goals, and link it from the control
  plane architecture map.
  Validation: `Get-Content docs\workstreams\durable-job-queue-and-resource-classes\WORKSTREAM.json | ConvertFrom-Json`; `git diff --check`.
  Evidence: `README.md`, `DESIGN.md`, `TODO.md`, `WORKSTREAM.json`,
  `docs/architecture/CONTROL_PLANE.md`,
  `docs/architecture/WORKSTREAM_LINKS.md`.
  Result: DONE. Lane opened from the proposed control-plane architecture
  candidate.
  Handoff: Continue with `DJRC-020`; keep execution in `nako-server` until a
  real multi-crate caller appears.

## M1 - Process-Local Resource Class Registry

- [x] DJRC-020 [owner=codex] [deps=DJRC-010] [scope=crates/nako-server/src/app/runtime.rs,crates/nako-server/src/app/composition.rs]
  Goal: Add a runtime resource class registry and route existing scan,
  metadata, and webhook permit pools through registry-owned classes.
  Validation: `cargo nextest run -p nako-server runtime_resource_class --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: Do not migrate all workers or expose new wire DTOs in this task.
  Existing service constructors should keep receiving `Arc<Semaphore>` clones
  from the centralized registry.
  Evidence: Runtime tests covering duplicate rejection, missing class rejection,
  sorted diagnostics, permit accounting, and app composition diagnostics.
  Result: DONE. Added `RuntimeResourceClassRegistry` in server runtime,
  registry-owned process-local classes for `disk.scan`, `metadata.shared`, and
  `network.webhook`, internal app diagnostics, and composition tests proving
  configured budgets flow into the registry.
  Handoff: `DJRC-030` should add the explicit durable-job-class to budget-class
  mapping before any scheduler starts claiming by priority.

## M2 - Durable Job Class To Budget Mapping

- [x] DJRC-030 [owner=codex] [deps=DJRC-020] [scope=crates/nako-server/src/app/runtime.rs,crates/nako-server/src/app/jobs.rs,crates/nako-server/src/app/metadata.rs,crates/nako-server/src/app/nfo.rs,crates/nako-server/src/app/addons]
  Goal: Add an explicit mapping from durable `job.resource_class` values to
  runtime budget classes without prefix inference.
  Validation: `cargo nextest run -p nako-server job_resource_class --no-fail-fast`; `cargo check -p nako-server --tests`; `cargo nextest run -p nako-server runtime_resource_class --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: Unknown resource classes must fail closed or be assigned through an
  explicit fallback policy, not silently bypass budgets.
  Evidence: Tests for known scan, metadata provider, NFO, artwork, addon, and
  webhook mappings.
  Result: DONE. Added a mapper from durable job kind/resource class to runtime
  budget class, routed immediate durable `spawn_job` callers through mapped
  budget classes, added registry entries for `artwork.ingest` and `addon.task`,
  and changed new addon task jobs to the fixed `addon.task` class while keeping
  legacy addon resource classes schedulable through the mapper.
  Handoff: `DJRC-040` can add scheduler admission without relying on ad hoc
  string prefix inference.

## M3 - Scheduler Admission Tracer Bullet

- [x] DJRC-040 [owner=codex] [deps=DJRC-030] [scope=crates/nako-server/src/app/job_runtime.rs,crates/nako-server/src/app/jobs.rs,crates/nako-server/src/app/tests/startup.rs]
  Goal: Introduce a typed scheduler admission loop for one existing durable job
  family while preserving leases, cancellation checkpoints, and typed executor
  ownership.
  Validation: `cargo nextest run -p nako-server job_scheduler --no-fail-fast`; `cargo nextest run -p nako-server job_runtime --no-fail-fast`; `cargo nextest run -p nako-server background_scan_job --no-fail-fast`; `cargo check -p nako-server --tests`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: The scheduler may select and admit work, but it must not become a
  generic JSON task dispatcher.
  Evidence: Tests showing budget-limited admission and no double execution.
  Result: DONE. Library scan background work now admits queued durable jobs
  through a typed scheduler path: it tries the scan budget first, claims one
  queued `LibraryScan` lease only when budget is available, runs the already
  leased job through `DurableJobRuntime`, and schedules a supervised follow-up
  admission pass after completion. Budget saturation leaves additional scan jobs
  queued instead of spawning tasks that wait on the semaphore.
  Handoff: `DJRC-050` should add persisted retry/backoff and queue pressure
  diagnostics without turning the scheduler into a generic JSON task runner.

## M4 - Retry, Backoff, And Queue Pressure

- [x] DJRC-050 [owner=codex] [deps=DJRC-040] [scope=crates/nako-core,crates/nako-db,crates/nako-server]
  Goal: Add explicit retry/backoff policy and redacted queue pressure
  diagnostics for at least network/provider work.
  Validation: repository contract tests plus focused server scheduler tests.
  Review: Persist enough retry state to survive restart; keep cancellation
  distinct from retryable failure.
  Evidence: DB/server tests and Admin-safe diagnostics coverage.
  Result: DONE. Added persisted durable job retry metadata
  (`attempt`, `max_attempts`, `retry_of_job_id`, `next_attempt_at`) to SQLite
  and PostgreSQL baselines, introduced `enqueue_job_retry` so retries create a
  new queued job row that copies the failed job input, and made lease claims
  skip queued jobs whose retry backoff is not due. Added redacted queue pressure
  summaries grouped by job kind, status, and resource class; diagnostics expose
  counts and retry timing only, not inputs, summaries, errors, paths, payloads,
  or secrets. Cancellation remains terminal and non-retryable through the retry
  source validation.
  Handoff: `DJRC-060` should close this lane or split broader priority,
  distributed scheduling, and remaining job-kind migration work.

## M5 - Closeout Or Split

- [x] DJRC-060 [owner=planner] [deps=DJRC-050] [scope=docs/workstreams/durable-job-queue-and-resource-classes]
  Goal: Close the lane or split remaining distributed scheduling, remote worker,
  child-process cancellation, and broader job-kind migration work.
  Validation: `verify-rust-workstream` records fresh gate evidence.
  Review: `review-workstream` has no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Result: DONE. Closeout review found no blocking findings. Fresh focused gates
  passed for resource registry, durable job class mapping, scheduler admission,
  runtime behavior, retry/backoff, queue pressure diagnostics, cross-crate
  compilation, formatting, JSON parsing, and whitespace. Priority policy,
  distributed scheduling, remote workers, addon process lifecycle,
  child-process cancellation, and broader job-kind scheduler migration were
  split as follow-ons, with
  `proposed:durable-job-priority-policy-and-scheduler-migration` as the next
  highest-leverage lane.
