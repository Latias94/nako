# M2 Watch Folder Admin Diagnostics

## Goal

Expose redaction-safe watch-folder runtime diagnostics through Admin surfaces so
operators can tell whether realtime library intake is idle, waiting for
stability, admitting a new scan, reusing queued/running scan work, suppressed by
host-owned writes, or backing off after discovery failures.

This builds on the internal `WatchFolderScanAdmissionStatus` diagnostic shipped
in the previous M2 slice.

## What I Already Know

- `GET /admin/v1/overview` already includes
  `startup.watch_folder_runtime` coverage diagnostics for started, disabled,
  unsupported-root, and missing-root libraries.
- Admin Web Overview already renders operator readiness, source fingerprint
  pressure, storage, and metadata panels.
- The current Admin overview watch-folder data is startup coverage only. It
  does not expose last tick intake counts, enqueue reason, admission status, or
  backoff status.
- `WatchFolderRuntimeTickDiagnostic` now contains:
  - `scan_admission_status`
  - `scan_job_id`
  - `reused_existing_scan`
  - `backoff_required`
  - `discovery_failures`
  - the pure `WatchFolderIntakePlan`
- Runtime tick diagnostics are currently returned to tests/runtime loops but
  are not retained for Admin reads.

## Assumptions

- The first useful Admin surface should remain read-only.
- Diagnostics must remain redaction-safe and bounded.
- This task should not introduce OS watcher events, schema migrations, or a new
  scan executor.
- It is acceptable for the first runtime diagnostic cache to be process-local,
  because watch-folder tick state is also process-local today.

## Requirements

- Provide an Admin-readable watch-folder runtime diagnostic summary for each
  configured library.
- Include coverage facts already available today:
  - library ID/name;
  - root scheme;
  - redacted root reference;
  - started/disabled/unsupported/missing status;
  - safe reason.
- Include last-tick facts for started libraries when a tick has run:
  - intake enqueue reason;
  - scan admission status;
  - admitted/reused scan job ID when present;
  - newly ready, inspecting, ready, suppressed, observed, and failure counts;
  - backoff flag;
  - discovery failure count and safe failure classes/messages.
- Distinguish `never_ticked` from a library that ticked and had no new work.
- Keep raw paths, Source Locators, StorageUri strings, fingerprints, etags,
  credentials, backend URLs, and raw provider/storage error text out of Admin
  DTOs, logs, mock data, and tests.

## Acceptance Criteria

- [ ] Admin DTOs expose a bounded watch-folder diagnostic summary.
- [ ] Runtime tick stores the most recent redaction-safe diagnostic per library
      without changing scan admission behavior.
- [ ] Admin overview or a focused Admin diagnostics route returns the latest
      watch-folder status for started libraries.
- [ ] `not_admitted`, `enqueued`, `reused_queued`, and `reused_running` remain
      distinguishable in Admin output.
- [ ] Libraries that never ticked are explicitly distinguishable from idle
      ticked libraries.
- [ ] Backend tests prove DTO serialization and redaction.
- [ ] Admin Web renders the new facts if the MVP includes UI.
- [ ] Specs document the diagnostic cache/DTO contract.

## Definition of Done

- Focused Rust tests pass.
- Focused Admin Web tests pass if UI is included.
- `cargo check -p nako-api -p nako-server --tests` passes if Admin DTOs change.
- `cargo fmt --all` and `git diff --check` pass.
- Trellis task context validates.
- Relevant `.trellis/spec/` docs are updated.

## Candidate MVP Options

### Option A: Backend Overview DTO Only

Extend `GET /admin/v1/overview` with watch-folder last-tick facts, plus backend
tests and spec updates. Admin Web can consume it later.

Pros:
- smallest backend slice;
- reuses existing Admin overview route and readiness context;
- useful for API/operator smoke immediately.

Cons:
- users of Admin Web still cannot see the facts without inspecting JSON.

### Option B: Backend Overview Plus Admin Web Panel

Extend `GET /admin/v1/overview` and add a compact Watch Folder panel to Admin
Web Overview using the existing overview data source.

Pros:
- complete operator-facing path;
- avoids adding a new route;
- keeps M2 observability visible in the existing readiness page.

Cons:
- touches backend DTOs, generated/admin web types, mock data, i18n, and UI
  tests in one slice.

### Option C: Dedicated Admin Watch-Folder Diagnostics Route

Add a new `GET /admin/v1/libraries/watch-folder/diagnostics` route and leave
overview coverage summary mostly unchanged.

Pros:
- cleaner long-term drilldown surface if diagnostics grow.

Cons:
- larger API route/inventory/contract surface;
- more likely to need generated contract updates and route documentation;
- overkill before we know the exact operator workflow.

## Decision (ADR-lite)

**Context**: Watch-folder runtime coverage already appears in Admin Overview,
but the new per-tick admission status is internal only. M2 needs reliability
diagnostics that operators can actually see without adding watcher behavior or
new execution paths.

**Decision**: Use Option B. Extend `GET /admin/v1/overview` with
redaction-safe watch-folder last-tick facts and add a compact Admin Web
Overview panel with aggregate counts plus a bounded per-library table.

**Consequences**:

- Backend and Web stay aligned in one operator-facing slice.
- The task touches Admin DTOs, generated/admin web types, mock data, i18n, and
  focused tests.
- A dedicated drilldown route remains deferred until diagnostics need
  pagination, history, filtering, or per-library actions.

Reasoning:
- M1/M2 product direction already uses Admin Overview as the operator readiness
  entry point.
- The backend already has watch-folder coverage in Overview; adding last-tick
  facts there is an incremental contract change.
- Showing the panel in Admin Web prevents an API-only diagnostic from becoming
  invisible to the actual operator journey.

## Out of Scope

- OS filesystem watcher integration.
- Remote storage watcher semantics.
- Durable persistence of tick history.
- New schema/migration.
- New scan/probe execution path.
- Automatic source duplicate relationship reconciliation.
- Incident bundle export unless the selected MVP explicitly includes it.

## Technical Notes

- Existing backend DTOs:
  - `crates/nako-api/src/admin.rs`
  - `AdminOverviewStartupSummary`
  - `AdminOverviewWatchFolderRuntimeSummary`
  - `AdminWatchFolderRuntimeCoverageDiagnostic`
- Existing backend route/mapping:
  - `crates/nako-server/src/http/admin.rs`
  - `admin_overview_response`
  - `admin_watch_folder_runtime_summary`
  - `watch_folder_runtime_coverage_gap`
- Existing runtime source:
  - `crates/nako-server/src/app/watch_folder_runtime.rs`
- Existing Admin Web overview:
  - `apps/admin-web/src/features/overview/OverviewPage.tsx`
  - `apps/admin-web/src/adminApi/mockData.ts`
  - `apps/admin-web/src/adminApi/types.ts`
- Relevant specs:
  - `.trellis/spec/nako-server/backend/directory-structure.md`
  - `.trellis/spec/nako-server/backend/quality-guidelines.md`
  - `.trellis/spec/nako-api/backend/index.md`
  - `.trellis/spec/admin-web/frontend/index.md`

## Open Questions

- None. MVP is Option B.
