# Repository Seam Deepening Milestones

Status: Completed
Last updated: 2026-05-17

## M39.0 Scope And Baseline

Outcome: M39 has an explicit workstream and chooses catalog hydration as the
first repository seam slice.

Exit evidence:

- Workstream docs exist under `docs/workstreams/repository-seam-deepening`.
- GOALS, ROADMAP, and workstream index name M39.
- The first slice excludes playback, NFO Round Trip, public API/SDK, and DB
  schema changes.

## M39.1 Catalog Hydration Port

Outcome: catalog hydration has a workflow port with a deeper interface.

Exit evidence:

- `hydrate_item_catalog` depends on `CatalogHydrationPort`.
- `SqliteStore` implements the port without schema changes.
- Fake-port tests cover hydration behavior without requiring SQLite.
- Existing SQLite-backed catalog hydration tests still pass.

## M39.2 Caller Narrowing

Outcome: metadata and NFO workflows no longer require the broad catalog/media/
search trait combination just to call hydration.

Exit evidence:

- Metadata refresh and hierarchy confirmation bounds use the new port.
- NFO import bounds use the new port.
- Focused metadata and NFO tests pass.

## M39.3 Closeout

Outcome: M39 proves one repository seam deepening slice and records follow-ons.

Exit evidence:

- Focused catalog, metadata, and NFO checks/tests passed.
- Workspace check and nextest gates passed.
- Follow-ons are kept out of M39 and should be opened as a new goal when the
  next repository seam is selected.
