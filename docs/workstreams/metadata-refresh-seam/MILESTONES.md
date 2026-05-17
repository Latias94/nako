# Metadata Refresh Seam Milestones

Status: Completed
Last updated: 2026-05-17

## M40.0 Scope And Baseline

Outcome: M40 has an explicit workstream and chooses metadata refresh as the
next repository seam slice after catalog hydration.

Exit evidence:

- Workstream docs exist under `docs/workstreams/metadata-refresh-seam`.
- GOALS, ROADMAP, and workstream index name M40.
- The first slice excludes provider breadth, NFO Round Trip, public API/SDK,
  playback, and DB schema changes.

## M40.1 Refresh Seam Audit

Outcome: current refresh, confirmation, provider mapping, raw cache, attempt,
and hydration dependencies are mapped to a concrete first port shape.

Exit evidence:

- `strategy.rs` and `confirmation.rs` dependency surfaces are reviewed.
- The chosen port hides a workflow, not merely a renamed repository method.
- Follow-ons are documented when they belong outside the first slice.

Status: completed for the first slice. `MetadataRefreshPort` owns the refresh
snapshot/commit workflow, `MetadataAttemptPort` owns attempt diagnostics, and
provider fetch remains outside the persistence port.

## M40.2 Metadata Refresh Port

Outcome: metadata refresh depends on a deeper workflow port where it currently
has to know too many repository details.

Exit evidence:

- A focused port is implemented in the right crate boundary.
- Existing refresh behavior remains unchanged.
- `CatalogHydrationPort` remains the catalog hydration boundary.

Status: completed for `crates/taru-metadata/src/strategy.rs`.

## M40.3 Fake-Port Behavior Test

Outcome: the refresh workflow can be tested without SQLite for the behavior
covered by the new port.

Exit evidence:

- A small fake adapter drives a focused behavior test.
- Existing SQLite-backed metadata tests still pass.

Status: completed. `strategy::port_tests` exercises refresh and hydration
ports without SQLite, and existing metadata tests continue to pass.

## M40.4 Closeout

Outcome: M40 proves one metadata refresh seam deepening slice and records
follow-ons.

Exit evidence:

- Focused metadata check and nextest gates passed.
- Workspace check and nextest gates passed.
- Provider runtime, metadata maintenance, and library scan/probe seams remain
  follow-on candidates instead of being mixed into this slice.
