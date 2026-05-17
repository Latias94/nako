# Catalog Hydration Lookup Deepening Design

Status: Completed
Last updated: 2026-05-17

## Problem

`CatalogHydrationPort` currently requires three implementation-shaped methods:

- load a catalog hydration snapshot;
- load a catalog hydration lookup containing match vectors;
- commit a graph replacement and search document.

This leaks too much `taru-catalog` implementation detail across the seam. A
metadata refresh fake port must know about `CatalogHydrationLookup` and build
empty person/genre/tag/collection/studio/image vectors just to say "hydration
was requested." The interface is therefore shallow: callers see nearly as much
process detail as the implementation.

## Target State

- Callers outside `taru-catalog` depend on a single workflow operation:
  hydrate this item from this metadata source.
- Snapshot, lookup, and commit types are no longer part of the public adapter
  surface unless a real caller proves they are needed.
- SQLite remains the production adapter for catalog hydration.
- Metadata and NFO workflows still request hydration, but their tests can fake
  that request without modeling catalog lookup internals.

## In Scope

- `CatalogHydrationPort` interface shape.
- `hydrate_item_catalog` orchestration and helper visibility.
- Metadata strategy fake-port tests that currently construct lookup vectors.
- Workstream and top-level docs.

## Out Of Scope

- SQLite schema changes.
- Public HTTP route behavior.
- SDK/OpenAPI/protocol contracts.
- Provider matching breadth.
- NFO codec/workflow changes beyond bound compatibility.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| External callers only need to request hydration and observe the summary. | High | `rg hydrate_item_catalog` shows metadata and NFO callers use only summary/error behavior. | If a caller needs commit internals, expose a test-only hook instead of widening the port. |
| Snapshot/lookup/commit are catalog implementation details. | High | Only catalog tests and metadata fake tests name them today. | If another crate genuinely owns part of hydration, split a narrower follow-on. |
| Preserving graph/search behavior is more important than changing resolver semantics. | High | M39 tests cover current hydration output. | Resolver changes should get their own behavior tests. |

## Architecture Direction

The deepened module is catalog hydration itself. Its interface should be:

```text
hydrate item catalog(item_id, metadata_source) -> CatalogHydrationSummary
```

The implementation can still perform snapshot, lookup, graph replacement, and
search projection internally. The adapter seam should not force callers to
understand those implementation stages.

## Closeout Condition

This lane can close when:

- non-catalog crates no longer import `CatalogHydrationLookup`,
  `CatalogHydrationSnapshot`, or `CatalogHydrationCommit`;
- metadata fake tests can prove hydration was requested through a simple
  summary-returning fake;
- existing catalog hydration tests still prove graph/search behavior;
- focused and workspace validation gates pass;
- docs record the new seam shape and follow-ons.
