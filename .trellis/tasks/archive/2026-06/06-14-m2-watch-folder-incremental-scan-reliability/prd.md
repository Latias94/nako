# M2 watch-folder incremental scan reliability first slice

## Goal

Productize the existing watch-folder runtime and incremental scan handoff as
the first M2 large-library reliability slice. The task should make local
realtime library updates observable, restart-safe at the runtime/admission
boundary, and testable under duplicate tick, suppression, and discovery failure
conditions without expanding into a broad watcher rewrite.

## Parent Decision

The parent planning task
`.trellis/tasks/06-14-06-14-m2-large-library-reliability-plan/` evaluated three
M2 candidates and selected watcher and incremental scan reliability as the
first executable slice.

## What I Already Know

* `WatchFolderRuntimeAppService` already owns startup coverage reporting,
  supervised runtime startup, and per-library `tick_library` polling.
* `AcquisitionIntakeAppService::discover_watch_folder_candidates` is already
  the stable-candidate observation authority.
* `nako_library::plan_watch_folder_intake` already defines the pure enqueue
  decision and only enqueues when `newly_ready_candidates > 0`.
* `LibraryScanAppService::admit_watch_folder_library_scan` already exists as
  the watch-folder scan admission path and should coalesce queued/running
  same-library scans.
* The server spec already defines a Watch-Folder Runtime Productization
  contract covering supervision, local-root eligibility, coverage diagnostics,
  suppression behavior, scan admission coalescing, and redaction-safe
  diagnostics.

## Requirements

* Keep the runtime under `RuntimeSupervisor`; do not add raw `tokio::spawn` or
  a second scan/probe executor.
* Keep local watch-folder eligibility narrow: persisted library,
  `realtime_monitor = true`, and first root parseable as local `StorageUri`.
* Preserve startup coverage diagnostics for started, disabled,
  unsupported-root, and missing-root libraries with redacted root references.
* Make runtime tick diagnostics sufficient for large-library operations:
  monitored state, intake plan, scan job id when admitted, coalescing/reuse
  state, and redaction-safe failure/backoff evidence where the current code is
  missing it.
* Runtime ticks may enqueue scans only through the existing library scan queue
  when the intake plan reports `newly_ready_candidates > 0`.
* Watch-folder scan admission must reuse an existing queued/running
  same-library scan and must not change manual/Admin scan behavior.
* Suppressed watch-folder entries must not advance stable observation evidence
  and must not enqueue scans.
* Discovery/storage failures must produce redaction-safe diagnostics/logging
  and use the bounded runtime error backoff rather than bypassing supervision.
* Add or deepen focused tests around the runtime/admission behavior instead of
  relying on release smoke tests.

## Acceptance Criteria

* [ ] A focused app test proves the runtime starts only for a persisted local
      realtime library and coverage diagnostics include skipped states with
      redacted root refs.
* [ ] A focused app/service test proves the first tick records inspecting
      candidates and enqueues no library scan.
* [ ] A focused app/service test proves the second unchanged tick reports
      newly ready candidates and admits exactly one `JobKind::LibraryScan`
      with resource class `disk.scan`.
* [ ] A focused app/service test proves a second unchanged tick with an
      existing queued/running same-library scan reuses the existing scan and
      creates no duplicate job.
* [ ] A focused app/service test proves repeated runtime ticks over a
      suppressed media URI do not enqueue a library scan.
* [ ] A focused failure test proves discovery/storage errors are redaction-safe
      and result in the runtime error-backoff path.
* [ ] Existing pure intake tests for first observation, repeated stable
      observation, changed observation reset, skip reasons, and redaction-safe
      counts remain green.

## Definition of Done

* Implementation stays inside existing server/library boundaries and follows
  the Watch-Folder Runtime Productization spec.
* Focused tests are added or strengthened for the acceptance criteria.
* At minimum, run focused nextest/check gates for the touched packages; broaden
  only if risk or failures require it.
* No schema migration, public API shape change, Admin Web route, or new storage
  backend behavior is introduced in this slice.
* Diagnostics and logs do not expose raw local paths, Source Locators,
  fingerprints, etags, credentials, backend URLs, or raw backend errors.

## Technical Approach

Treat this as a reliability hardening pass over existing code, not a new
watcher architecture.

1. Inspect `watch_folder_runtime`, `acquisition_intake`, `jobs`, and existing
   app tests to find the smallest missing reliability/diagnostic gaps.
2. Prefer pure diagnostic structs and focused app-service tests before adding
   any new runtime behavior.
3. Keep scan execution delegated to the durable library scan queue through the
   existing watch-folder admission path.
4. If failure/backoff evidence is added, make it redaction-safe and testable
   without exposing raw paths or backend details.

## Decision (ADR-lite)

**Context**: M2 needs a first user-visible reliability slice after M1 smoke
coverage, but broad watcher, VFS repair automation, or durable job platform work
would create too much scope.

**Decision**: Start by hardening the existing local watch-folder runtime,
incremental stable-candidate intake, and scan-admission coalescing behavior.

**Consequences**: This should improve day-to-day large local library updates
and operator evidence while leaving remote watcher semantics, automatic VFS
repair policy, generic job drilldown UI, and schema/API expansion for later M2
slices.

## Out of Scope

* Remote filesystem watcher semantics or assumptions for WebDAV/S3-like roots.
* New schema migrations or persistent watcher state.
* New public API fields, Admin API routes, or Admin Web UI.
* Automatic VFS cache repair enqueue/execution policy.
* Generic durable job retry/drilldown surfaces.
* Rewriting library scan, probe, VFS, or metadata pipelines.
* Copying implementation details from GPL reference projects.

## Technical Notes

Primary code areas the implementer should inspect:

* `crates/nako-server/src/app/watch_folder_runtime.rs`
* `crates/nako-server/src/app/acquisition_intake.rs`
* `crates/nako-server/src/app/jobs.rs`
* `crates/nako-server/src/app/tests/startup.rs`
* `crates/nako-server/src/app/tests/acquisition_intake.rs`
* `crates/nako-library/src/intake.rs`

Spec and architecture anchors:

* `.trellis/spec/nako-server/backend/directory-structure.md`
  `Scenario: Watch-Folder Runtime Productization`
* `.trellis/spec/nako-server/backend/quality-guidelines.md`
* `.trellis/spec/nako-library/backend/quality-guidelines.md`
* `docs/architecture/LIBRARY_PIPELINE.md`
* `docs/architecture/CONTROL_PLANE.md`
