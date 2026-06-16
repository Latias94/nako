# Media library scan job posture readiness

## Goal

Deepen the Admin operator readiness Media Library scan check so a self-hosted
operator can tell whether configured Media Libraries have successful,
failed, or pending `LibraryScan` durable job posture. This closes the gap where
the readiness view could say the Media Library scan area was ready even when a
configured library had never completed a scan or had only a failed scan job.

## Requirements

- Reuse existing durable `JobKind::LibraryScan` rows and repository APIs; do not
  add schema or new API DTOs in this slice.
- Compute a bounded aggregate scan posture across configured libraries:
  - queued/running scan work means scan work is pending;
  - failed scan work without a newer/same-library succeeded scan means repair
    pressure;
  - configured libraries without any successful scan job mean scan work is
    pending.
- Keep existing higher-priority readiness failures intact:
  - no configured Media Library remains unavailable;
  - runtime/source-hash repair pressure still reports repair pressure;
  - source-hash pending work still reports pending work;
  - watch-folder coverage gaps remain checked before a final ready result.
- Return only existing `AdminOperatorReadinessCheck` fields with safe fixed
  `source_reason` values such as `failed_library_scan` or
  `library_scan_never_completed`. Do not expose raw paths, source locators,
  input JSON, summaries, filenames, local host details, or job errors.
- Keep HTTP handlers thin: app/service code owns job aggregation, Admin overview
  maps the aggregate into readiness.

## Acceptance Criteria

- [ ] A clean configured library with no succeeded `LibraryScan` job reports
      Media Library scan degraded with `ScanWorkPending` and source reason
      `library_scan_never_completed`.
- [ ] A configured library with a failed `LibraryScan` job and no successful
      scan reports degraded with `ScanRepairPressure` and source reason
      `failed_library_scan`.
- [ ] Queued/running `LibraryScan` jobs report degraded with `ScanWorkPending`
      and source reason `library_scan_pending`.
- [ ] A configured library with a succeeded `LibraryScan` job and no other scan
      pressure still reports `MediaLibraryConfigured`.
- [ ] Overview responses and tests prove raw job input/error/source material is
      not exposed.

## Definition of Done

- Focused server tests cover no-success, failed, pending, and succeeded scan
  posture.
- `cargo fmt --all -- --check`, `cargo check -p nako-server --tests`, and a
  focused nextest filter for Admin overview scan readiness pass or failures are
  recorded.
- Trellis task validates with curated implementation/check context.

## Technical Approach

Add a small app-level aggregate on `LibraryScanAppService` that lists configured
libraries and bounded pages of existing `LibraryScan` jobs. The aggregate stays
count-based and redaction-safe. Admin overview calls that aggregate and extends
`media_library_scan_readiness_check` with the posture before returning ready.

## Decision (ADR-lite)

**Context**: The durable job table already records `LibraryScan` state, status,
library binding, and timestamps. Adding a separate scan-status table before a
product query proves it is needed would expand schema and migration scope.

**Decision**: Reuse durable `LibraryScan` jobs as the first operator-readiness
source of truth and keep the output as existing readiness check fields.

**Consequences**: This is a pragmatic first step that improves operator
visibility without new storage contracts. Later per-library diagnostics can add
a richer paginated surface if operators need drilldown beyond the Admin Jobs
route.

## Out of Scope

- New public/Admin DTOs, generated contract changes, or Admin Web rendering.
- Schema migrations or persistent per-library scan status records.
- Changing scan execution, scheduler ordering, watcher behavior, or source hash
  triggering.
- Exposing job input, summaries, errors, Source Locators, raw roots, or local
  paths through overview readiness.

## Technical Notes

- Relevant plan: `docs/plans/2026-06-16-001-feat-backend-self-hosted-maturity-execution-plan.md`.
- Relevant architecture: `docs/architecture/CONTROL_PLANE.md` and
  `docs/architecture/LIBRARY_PIPELINE.md`.
- Existing readiness logic: `crates/nako-server/src/http/admin.rs`.
- Existing scan durable job service: `crates/nako-server/src/app/jobs.rs`.
- Existing tests already cover source-hash pending/failed readiness; this task
  adds direct `LibraryScan` posture coverage.
