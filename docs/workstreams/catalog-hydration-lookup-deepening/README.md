# Catalog Hydration Lookup Deepening

Status: Completed
Last updated: 2026-05-17

## Why This Lane Exists

M39 extracted `CatalogHydrationPort`, but the port still exposes
implementation-shaped snapshot, lookup, and commit steps. The most problematic
part is `CatalogHydrationLookup`: adapters and fake tests must construct
person, genre, tag, collection, studio, and image match vectors even when they
only want to prove that a workflow requested catalog hydration.

This workstream deepens the seam so callers ask for catalog hydration as one
workflow operation. The lookup machinery stays inside `nako-catalog`, where the
catalog graph and search projection logic live.

## Relevant Authority

- Related workstreams:
  - `docs/workstreams/repository-seam-deepening/`
  - `docs/workstreams/metadata-refresh-seam/`
  - `docs/workstreams/durable-job-recovery/`
- Code:
  - `crates/nako-catalog/src/lib.rs`
  - `crates/nako-metadata/src/strategy.rs`
  - `crates/nako-metadata/src/confirmation.rs`
  - `crates/nako-nfo/src/import.rs`

## Scope

- Make `CatalogHydrationPort` expose a workflow-level hydrate operation.
- Hide snapshot, lookup, and commit details from external adapters and tests.
- Preserve current catalog graph and search projection behavior.
- Keep SQLite-backed behavior tests passing.

## Non-Goals

- No database schema changes.
- No public HTTP API, SDK, or client contract changes.
- No metadata provider breadth.
- No NFO round-trip preservation work.
- No catalog domain model redesign beyond this seam.

## Closeout

M42 shipped a workflow-level `CatalogHydrationPort` and narrowed the fake test
surface in metadata. Snapshot, lookup, and commit mechanics remain inside
`nako-catalog`, while non-catalog callers request hydration as one workflow
operation.
