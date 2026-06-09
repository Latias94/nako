# refactor: tighten catalog search access pagination

## Goal

Move Public Catalog `/search` access filtering out of the server app loop and
into database-backed search document projection, while keeping `nako-search`
as a transport-free pure scoring layer.

## What I already know

* `nako-search` owns `SearchDocument`, `SearchQuery`, `SearchHit`, and
  deterministic in-memory scoring.
* SQLite/PostgreSQL `SearchIndex::search` currently read all
  `search_documents`, then call `evaluate_search_documents`.
* `CatalogAppService::search_accessible_items` currently requests pages of
  global search hits, batches `list_accessible_media_items_by_ids`, skips
  inaccessible hits in the app loop, and repeats until enough visible hits are
  found.
* This preserves redaction but leaves an access-after-search path and keeps
  pagination compensation in the server app service.
* Existing adapter-local access helpers can add the same media item Library
  Access predicate used by `/items`, relation items, and aggregate browse
  projections.

## Assumptions

* Do not add access concepts to `nako-search`; it remains pure scoring.
* Do not change public `/search` route shape or response DTOs.
* Do not add schema or migration changes.
* A database-backed accessible search projection may still evaluate the
  accessible candidate documents in memory for this slice; the immediate win is
  that inaccessible documents no longer consume search pagination slots or drive
  repeated server-side access batches.
* Administrator principals keep existing search visibility over all indexed
  documents.

## Requirements

* Add a `nako-db` accessible search method for SQLite/PostgreSQL backends.
* Apply Library Access before `evaluate_search_documents` and before final
  search pagination.
* Preserve search scoring, facet parsing, score ordering, and DTO shape.
* Keep server app service as a thin mapper from accessible hits to accessible
  media item DTOs.
* Add backend-agnostic repository contract coverage for hidden-hit page holes.
* Keep focused HTTP route coverage for `/search`.

## Acceptance Criteria

* [x] `CatalogAppService::search_accessible_items` no longer loops through
  global search pages to compensate for inaccessible hits.
* [x] SQLite and PostgreSQL accessible search paths filter search documents by
  Library Access before scoring/pagination.
* [x] Backend-agnostic DB contract proves a hidden top search hit does not
  consume the first visible search page.
* [x] Existing `/search` HTTP regression remains green.
* [x] Focused DB/server checks and `git diff --check` pass.

## Definition of Done

* Tests added/updated where behavior is observable.
* SQLite and PostgreSQL adapter behavior stays in parity.
* Trellis task is archived and session journal is updated after commit.

## Out of Scope

* Replacing in-memory `nako-search` scoring with SQL FTS or external search.
* Changing public OpenAPI/client protocol.
* Adding total-count search metadata.
* Changing search document schema or projection version.
* Refactoring addon/metadata tests that call the raw `SearchIndex::search`.

## Technical Notes

* Relevant specs:
  * `.trellis/spec/nako-search/backend/index.md`
  * `.trellis/spec/nako-db/backend/database-guidelines.md`
  * `.trellis/spec/nako-db/backend/quality-guidelines.md`
  * `.trellis/spec/nako-core/backend/database-guidelines.md`
  * `.trellis/spec/nako-server/backend/http-api-patterns.md`
  * `.trellis/spec/nako-server/backend/database-guidelines.md`
  * `.trellis/spec/nako-server/backend/quality-guidelines.md`
  * `.trellis/spec/guides/cross-layer-thinking-guide.md`
  * `.trellis/spec/guides/code-reuse-thinking-guide.md`
* Architecture map: `docs/architecture/STATE_ACCESS.md`.
* No external research needed; this task follows existing repository-backed
  browse/access patterns.
