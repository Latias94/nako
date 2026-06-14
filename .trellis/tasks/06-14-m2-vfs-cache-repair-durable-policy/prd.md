# M2 VFS cache repair durable execution policy

## Goal

Make VFS cache repair move from explicit manual/automation commands toward a
safe, operator-controlled recurring policy. The next slice should periodically
discover eligible unresolved repair targets and enqueue durable
`VfsCacheRepair` jobs through the existing app-service authority, while keeping
execution delegated to the existing disk-scan scheduler and preserving all
redaction and non-destructive boundaries.

## What I Already Know

* M2's first slice hardened watch-folder and incremental scan reliability.
* VFS cache repair already has redaction-safe diagnostics, target previews,
  manual target refresh, durable job contract, internal enqueue, executor,
  scheduler integration, retry, Admin manual commands, Admin Jobs diagnostics,
  dry-run automation planning, and explicit Admin automation enqueue commands.
* `docs/architecture/STORAGE_VFS.md` and `docs/architecture/CONTROL_PLANE.md`
  both identify recurring automatic VFS cache repair scheduling/execution
  policy as a follow-on.
* Existing repair jobs use `JobKind::VfsCacheRepair` with resource class
  `storage.vfs.cache_repair`, mapped to the `disk.scan` runtime budget.
* Existing `enqueue_vfs_cache_repair_automation` is an explicit command that
  queues eligible targets and does not execute jobs or touch storage backends.
* Existing repair execution already runs through the disk-scan scheduler via
  `schedule_queued_library_scans`, not a dedicated repair worker.
* Existing config already uses disabled-by-default scheduled runtime patterns,
  for example `AddonEventSchedulerConfig`.

## Assumptions

* The M2 next step should not repeat manual enqueue/execute/retry work that is
  already shipped.
* A recurring policy must be disabled by default and explicitly configured.
* The recurring runtime may enqueue repair jobs but must not directly refresh
  cache, purge/delete/invalidate cache entries, mutate backend configuration,
  write library files, or bypass durable jobs.
* Execution should continue through the existing durable job scheduler and
  `disk.scan` budget.

## Requirements

* Add a narrowly scoped recurring VFS cache repair automation policy surface
  that can be configured independently of manual Admin commands.
* MVP scope is confirmed as automatically recurring enqueue of
  `refresh_cache` repair jobs only. The policy must not add destructive repair
  actions or a second execution path.
* Keep the recurring policy disabled by default.
* Reuse the existing dry-run planner and explicit automation enqueue command as
  the only enqueue authority for eligible targets.
* Reuse existing durable queue idempotency so queued/running repair jobs for the
  same target are not duplicated across ticks.
* Keep repair execution delegated to the existing disk-scan scheduler and
  `VfsCacheRepair` executor path.
* Run the recurring policy under `RuntimeSupervisor`; do not add raw
  `tokio::spawn` loops or a second repair executor.
* Include bounded interval and error-backoff behavior so storage/listing
  failures cannot hot-loop.
* Emit redaction-safe runtime diagnostics: enabled/disabled state, eligible and
  blocked target counts, enqueued/already-queued counts, last success/failure
  summary, and boundary flags.
* Preserve all existing redaction rules: no raw `StorageUri`, local path,
  backend URL, credential, target ref, URI digest, etag, fingerprint, cache
  payload, job input JSON, or raw backend error in logs, diagnostics, or
  persisted runtime evidence.

## Acceptance Criteria

* [x] Config parsing proves the recurring VFS cache repair policy defaults to
      disabled and accepts explicit interval/error-backoff settings.
* [x] Startup/runtime coverage proves the recurring policy starts only when
      enabled and is reported as disabled otherwise.
* [x] A focused app/runtime test proves one recurring tick reuses the existing
      automation planner/enqueue command and creates durable repair jobs for
      eligible refresh-cache targets.
* [x] A focused app/runtime test proves a second tick over the same unresolved
      target reports `already_queued` and does not create duplicate jobs.
* [x] A focused app/runtime test proves disabled policy, non-refresh targets,
      and blocked targets do not enqueue repair jobs.
* [x] A focused failure test proves planner/enqueue errors use bounded backoff
      and expose only redaction-safe diagnostics/log messages.
* [x] Existing manual VFS repair enqueue, execute, retry, scheduler, and Admin
      command tests remain green.

## Definition of Done

* The implementation stays inside existing server/storage/control-plane seams.
* No schema migration, new storage backend behavior, cache purge/delete/
  invalidation, backend configuration mutation, library file write, or public
  client API change is introduced.
* Admin API changes, if any, are limited to redaction-safe diagnostics or
  startup status facts and use existing Admin auth boundaries.
* Focused `nako-server` tests cover config, startup/runtime, recurring enqueue,
  idempotency, blocked targets, and redaction-safe failures.
* Relevant architecture/spec notes are updated if a new recurring-runtime
  pattern is codified.

## Technical Approach

Use a conservative recurring enqueue runtime:

1. Add a disabled-by-default config shape similar to the existing addon event
   scheduler config, with interval and error-backoff settings.
2. Add a server app/runtime component that periodically calls the existing
   `plan_vfs_cache_repair_automation` and
   `enqueue_vfs_cache_repair_automation` app-service methods with
   `enabled = true`.
3. Record redaction-safe per-tick diagnostics and startup coverage.
4. Leave actual job execution to `schedule_queued_library_scans` and the
   existing `VfsCacheRepair` durable executor.
5. Add focused tests before broad gates.

## Decision (ADR-lite)

**Context**: VFS cache repair already has manual durable repair commands and an
explicit automation enqueue command. The remaining M2 gap is recurring
operator-controlled policy, not another executor or Admin manual endpoint.

**Decision**: Implement the next slice as a disabled-by-default supervised
recurring enqueue policy that reuses existing planner/enqueue/executor
authorities.

**Consequences**: The task improves remote/cache reliability without widening
mutation semantics. It intentionally leaves destructive repair actions,
backend configuration workflows, realtime incident bundles, and broad job UI
work as later M2 follow-ons.

**MVP confirmation**: Proceed with automatic periodic enqueue of refresh-cache
repair jobs only. The runtime remains disabled by default and execution remains
delegated to the existing durable disk-scan scheduler.

## Out of Scope

* Cache purge/delete/invalidation semantics.
* Backend configuration mutation or credential repair workflows.
* Library file writes.
* A dedicated VFS repair executor or resource class outside the existing
  `disk.scan` budget mapping.
* New public API routes.
* Broad Admin Web UI work.
* Automatic Source Duplicate Relationship reconciliation.
* Copying implementation details from GPL reference projects.

## Technical Notes

Architecture/spec anchors:

* `docs/architecture/STORAGE_VFS.md`
* `docs/architecture/CONTROL_PLANE.md`
* `.trellis/spec/nako-server/backend/index.md`
* `.trellis/spec/nako-server/backend/quality-guidelines.md`
* `.trellis/spec/nako-vfs/backend/index.md`
* `.trellis/spec/nako-vfs/backend/error-handling.md`

Likely code areas:

* `crates/nako-server/src/config.rs`
* `crates/nako-server/src/app/startup.rs`
* `crates/nako-server/src/app/runtime.rs`
* `crates/nako-server/src/app/storage.rs`
* `crates/nako-server/src/app/jobs.rs`
* `crates/nako-server/src/app/tests/storage.rs`
* `crates/nako-server/src/app/tests/startup.rs`

Focused gates likely needed:

* `cargo fmt --all -- --check`
* `cargo check -p nako-server --tests`
* `cargo check -p nako-api -p nako-server --tests` if Admin DTOs change
* `cargo nextest run -p nako-server vfs_cache_repair_automation --no-fail-fast`
* `cargo nextest run -p nako-server vfs_cache_repair_scheduler --no-fail-fast`
* `cargo nextest run -p nako-server startup --no-fail-fast` or a narrower
  runtime/startup filter once test names are known

## Implementation Evidence

Completed in this task:

* Added disabled-by-default `VfsCacheRepairAutomationRuntimeConfig`.
* Added `VfsCacheRepairAutomationRuntimeAppService` as a supervised recurring
  enqueue runtime.
* Runtime ticks call the existing storage automation enqueue authority with
  `enabled = true` and low priority, then ask the existing disk-scan scheduler
  to schedule queued/already queued repair work.
* Added startup report coverage for
  `vfs_cache_repair_automation_started`.
* Kept execution inside the existing `VfsCacheRepair` durable executor and
  `disk.scan` scheduler path; no storage backend mutation, purge/delete/
  invalidation, backend configuration mutation, library file writes, schema
  migration, public API route, or second repair executor was added.
* Updated architecture/spec notes for the recurring enqueue runtime boundary.

Verification run:

* `cargo fmt --all`
* `cargo check -p nako-server --tests`
* `cargo nextest run -p nako-server vfs_cache_repair_automation --no-fail-fast`
* `cargo nextest run -p nako-server vfs_cache_repair_scheduler --no-fail-fast`
* `cargo nextest run -p nako-server startup --no-fail-fast`
* `git diff --check`
