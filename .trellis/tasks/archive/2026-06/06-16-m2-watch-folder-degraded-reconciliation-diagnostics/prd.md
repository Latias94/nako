# M2 Watch-Folder Degraded Reconciliation Diagnostics

## Goal

Deepen the watch-folder runtime follow-on by turning unreliable watch-folder outcomes into explicit, redaction-safe Admin diagnostics and scan-handoff states. The first useful slice should tell an operator whether watch-folder intake is healthy, suppressed, degraded, or waiting for reconciliation, without changing the watcher execution model or adding a new scan executor.

## What I Already Know

- `watch_folder_runtime.rs` already productized the supervised watch-folder runtime and the internal `scan_admission_status` contract.
- The previous M2 slice already exposed watch-folder startup coverage and last-tick facts through the Admin overview path.
- `docs/architecture/LIBRARY_PIPELINE.md` still marks watcher/debounce as weak and points to `library-watcher-and-media-intake-stability`.
- `docs/architecture/STATE_ACCESS.md` and `docs/architecture/CONTROL_PLANE.md` already frame bounded read models and redaction-safe diagnostics.
- Existing candidate workstream notes call out degraded and reconciliation-pending diagnostics as the remaining gap after runtime productization.
- Current operator-visible output does not yet give a first-class typed state for degraded or unresolved watch-folder outcomes beyond the internal admission status and counts.

## Assumptions

- The first slice should stay backend-first.
- The existing Admin overview/read model is the right operator surface unless later evidence proves a separate drilldown route is necessary.
- Reconciliation and degradation states should be modeled as typed diagnostics, not inferred from free-form messages.
- A new runtime worker or scan executor would be the wrong shape for this gap.

## Requirements

- Add a typed watch-folder runtime outcome summary that can represent healthy, degraded, suppression-blocked, and reconciliation-pending states.
- Keep the summary redaction-safe: no raw paths, Source Locators, etags, fingerprints, credentials, backend URLs, or raw provider/storage error text.
- Preserve existing scan-admission behavior; this slice only changes diagnostics and handoff visibility.
- Reuse the existing supervised runtime and scan-admission path; do not add a new worker, raw `tokio::spawn`, or direct scan/probe execution.
- Expose the new state through the existing Admin overview/read model contract.
- Distinguish `never_ticked` from `ticked but idle` and from `degraded` or `reconciliation_pending`.
- Preserve the existing stable-candidate and suppression semantics.

## Acceptance Criteria

- [x] A started watch-folder library can report a typed healthy, degraded, or reconciliation-pending state.
- [x] A suppressed or blocked watch-folder tick reports a distinct state, not just `NotAdmitted`.
- [x] Discovery or storage failures map to safe degraded states with bounded redaction-safe reason fields.
- [x] The Admin overview response includes the new watch-folder state and scan-handoff facts.
- [x] Tests prove `never_ticked`, idle, and degraded/reconciliation-pending are distinguishable.
- [x] Existing scan admission tests stay green.
- [x] Admin contract generation remains synchronized if DTOs change.

## Verification Evidence

- `cargo nextest run -p nako-server watch_folder_runtime --no-fail-fast`
  passed with 13 tests.
- `cargo nextest run -p nako-server media_library_scan_readiness_reports_watch_folder_runtime_coverage_gap --no-fail-fast`
  passed.
- `cargo nextest run -p nako-api admin_overview_response_serializes_safe_summary_fields --no-fail-fast`
  passed.
- `cargo check -p nako-api -p nako-server --tests` passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed.
- `python ./.trellis/scripts/task.py validate '.trellis/tasks/06-16-m2-watch-folder-degraded-reconciliation-diagnostics'`
  passed.

## Definition of Done

- Focused `nako-server` tests pass.
- `cargo check -p nako-api -p nako-server --tests` passes if Admin DTOs change.
- `cargo fmt --all -- --check` and `git diff --check` pass.
- Trellis task context validates.
- Relevant spec notes are updated if the diagnostic contract changes.

## Technical Approach

Use the existing watch-folder runtime and Admin overview path:

1. Extend the watch-folder runtime diagnostic model with a typed outcome state instead of adding another worker path.
2. Derive the state from the current runtime/intake/suppression boundary so failures, suppression, and reconciliation intent stay aligned with existing behavior.
3. Surface the state through the existing Admin overview read model so operators can see it without a separate UI workflow in this slice.
4. Add focused server/runtime and API contract tests around state transitions, scan handoff, and redaction.

## Decision (ADR-lite)

**Context**: The runtime already knows when it enqueues, suppresses, or backs off. What it still lacks is a typed operator-facing state for unreliable or unresolved watch-folder behavior.

**Decision**: Deepen the existing Admin overview and watch-folder diagnostic path with a typed degraded/reconciliation-pending summary, using the current runtime and scan-admission boundary.

**Consequences**: Operators get an explicit signal for watch-folder unreliability without opening a new worker or UI path. Dedicated drilldown or Web polish can come later if the new state proves useful.

## Out of Scope

- OS filesystem watcher integration.
- A new scan executor or worker loop.
- Broad Admin Web redesign or a new UI surface.
- Persisted long-term history of watcher events.
- Automatic source duplicate reconciliation.
- Schema migration.

## Technical Notes

- Architecture: `docs/architecture/LIBRARY_PIPELINE.md`, `docs/architecture/STORAGE_VFS.md`, `docs/architecture/CONTROL_PLANE.md`, `docs/architecture/STATE_ACCESS.md`, `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md`
- Spec: `.trellis/spec/nako-server/backend/index.md`, `.trellis/spec/nako-server/backend/directory-structure.md`, `.trellis/spec/nako-server/backend/quality-guidelines.md`, `.trellis/spec/nako-library/backend/index.md`, `.trellis/spec/nako-library/backend/quality-guidelines.md`, `.trellis/spec/nako-api/backend/index.md`, `.trellis/spec/nako-api/backend/quality-guidelines.md`
- Prior task refs: `.trellis/tasks/archive/2026-06/06-14-m2-watcher-incremental-scan-reliability/prd.md`, `.trellis/tasks/archive/2026-06/06-14-06-14-m2-watch-folder-admin-diagnostics/prd.md`, `.trellis/tasks/archive/2026-06/06-16-backend-readiness-control-plane-audit/prd.md`
- Likely code areas: `crates/nako-server/src/app/watch_folder_runtime.rs`, `crates/nako-server/src/app/acquisition_intake.rs`, `crates/nako-server/src/app/watch_folder_suppression.rs`, `crates/nako-server/src/http/admin.rs`, `crates/nako-api/src/admin/intake.rs`
