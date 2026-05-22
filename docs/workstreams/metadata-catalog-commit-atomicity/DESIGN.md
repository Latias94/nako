# Metadata Catalog Commit Atomicity

Status: Completed
Last updated: 2026-05-18

## Why This Lane Exists

Metadata refresh currently updates several durable records through ordered
workflow calls: canonical item metadata and raw provider response, provider
mapping acceptance, library item confirmation, catalog graph hydration, and
search projection. Some lower-level calls are transactional, but the workflow as
a whole still exposes ordering details to callers.

The first concrete gap was catalog hydration itself: `replace_item_catalog_graph`
and `SearchIndex::upsert` were separate writes. If the first write succeeded and
the second failed, the Catalog Item Graph and Search Projection could disagree.

The second concrete gap was metadata refresh persistence. Canonical Metadata,
Provider Raw Response, Provider Subject, Provider Mapping, and Library Item
State confirmation were previously persisted through separate workflow-level
repository calls. A failure after an earlier write could leave provider state or
library confirmation behind the accepted Canonical Metadata.

## Relevant Authority

- `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
- `docs/adr/0011-normalized-catalog-graph-and-search-projection.md`
- `docs/adr/0019-server-architecture-hardening-boundaries.md`
- `docs/workstreams/architecture-review-followups/DESIGN.md`
- `docs/workstreams/repository-seam-deepening/`
- `docs/workstreams/metadata-refresh-seam/`
- `docs/workstreams/catalog-hydration-lookup-deepening/`

## Problem

The current interface is still too shallow for consistency-sensitive metadata
workflows. A caller must know that:

- metadata refresh commit happens before catalog hydration;
- provider mapping acceptance and library item confirmation happen in separate
  repository calls;
- catalog graph replacement happens before search projection;
- failures after an earlier write may leave projections stale.

That knowledge belongs behind a deeper commit interface, not spread across
metadata, catalog, NFO, and server orchestration code.

## Target State

- Catalog hydration writes Catalog Item Graph and Search Projection records
  atomically.
- Metadata refresh persists Canonical Metadata, Provider Raw Response, accepted
  Provider Mapping, Provider Subject, and Library Item State confirmation
  through one explicit commit unit.
- Catalog hydration remains a separate workflow step for now, but its internal
  Catalog Item Graph and Search Projection writes are atomic after MCC-020.
- Tests exercise the commit interface rather than the internal ordering.
- Existing broad repository traits remain only where query/admin surfaces still
  need them.

## In Scope

- Add a catalog hydration commit interface that persists graph replacement and
  search projection together.
- Implement the SQLite adapter transaction for that commit.
- Update catalog hydration to use the new commit interface.
- Add focused tests proving both graph and search projection are visible after
  hydration.
- Add a metadata refresh persistence commit interface that folds Canonical
  Metadata, Provider Raw Response, Provider Subject, Provider Mapping, and
  Library Item State confirmation into one SQLite transaction.
- Remove the shallow two-record metadata refresh commit path so workflow code
  cannot bypass provider acceptance and library confirmation.
- Document remaining follow-on work for metadata refresh plus catalog hydration
  closeout.

## Out Of Scope

- New metadata providers.
- NFO merge policy redesign.
- Public Client API changes.
- Addon behavior.
- Folding catalog hydration into the same metadata refresh persistence
  transaction.
- Search adapter replacement with FTS/Tantivy/Meilisearch.
- Broad repository trait reshuffling unrelated to the commit path.
- Schema migrations unless the first slice proves the current schema cannot
  express atomic graph/search commit.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| SQLite can atomically write graph replacement and search projection in one transaction. | High | `crates/nako-db/src/catalog.rs` already writes graph replacement in one transaction; `crates/nako-db/src/search.rs` upserts search documents with one SQL statement. | Add a schema or adapter-specific follow-up if search moves out of SQLite. |
| Search projection is currently local to SQLite for the production adapter. | High | `SqliteStore` implements `SearchIndex`; external search adapters are future ADR 0011 work. | If a second adapter appears, split an outbox/projection lane. |
| Metadata refresh persistence can fold Provider Subject, Provider Mapping, and Library Item State confirmation into the same SQLite transaction without a schema migration. | High | Existing tables have the necessary foreign keys and repository adapters already own SQLite write helpers. | Split adapter-specific transaction helpers or add schema support if another backend cannot express the same unit. |
| Catalog hydration should remain outside the MCC-030 transaction. | Medium | Hydration performs graph lookup and projection construction across catalog modules; MCC-020 already made the graph/search write atomic. | MCC-040 should decide whether a prepared catalog commit belongs in a larger metadata refresh unit or in an outbox/projection lane. |

## Architecture Direction

Deepen the Catalog Hydration Module first. The useful interface is not
`replace graph` plus `upsert search`; it is `commit hydrated catalog`, because
callers want one observable result: item-facing metadata has a consistent graph
and search projection.

The SQLite Adapter should own the transaction. Workflow crates should not build
SQL transaction ordering at the app level.

After MCC-030, metadata refresh has a deeper persistence commit interface:
workflow code prepares provider acceptance data, and the SQLite Adapter owns the
single durable commit. The adapter also reads the current Library Item State
rows inside that transaction before confirming them, so callers do not pass a
stale list of library IDs.

Catalog hydration still happens after metadata refresh. The remaining design
question is whether this is sufficient because hydration's own write is atomic,
or whether a future lane should turn metadata refresh plus prepared catalog
hydration into a larger unit of work or event-driven projection pipeline.

## Closeout Condition

This lane can close when:

- catalog hydration graph/search commit is atomic for SQLite;
- metadata refresh follow-up unit-of-work work is either completed or split;
- targeted and workspace-appropriate Rust gates pass;
- `EVIDENCE_AND_GATES.md` records fresh validation;
- `WORKSTREAM.json` and `HANDOFF.md` reflect the final status.

## Closeout Decision

This lane closes after MCC-040.

The remaining question of whether metadata refresh and prepared catalog
hydration should be one larger commit unit is intentionally not continued here.
The current implementation removed the two concrete partial-write windows this
lane targeted:

- Catalog Item Graph replacement and Search Projection upsert commit together.
- Metadata refresh persistence commits Canonical Metadata, Provider Raw
  Response, Provider Subject, accepted Provider Mapping, and Library Item State
  confirmation together.

Crossing into metadata-refresh-plus-catalog-hydration would require a separate
projection pipeline or prepared projection design. That work should be routed
through `architecture-review-followups` rather than extending this lane.
