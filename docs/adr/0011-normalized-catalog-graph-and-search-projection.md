# 0011: Normalize Catalog Graph and Project Search Documents

Status: accepted

## Context

Provider metadata, NFO imports, item detail pages, actor pages, tag filters, and
search cannot all depend on scanning `MediaItem.metadata_json`. Keeping people,
genres, tags, collections, studios, and images only as embedded JSON would make
queries slow and force each provider to duplicate relationship handling.

## Decision

Nako stores normalized catalog graph records beside canonical item metadata.
`CanonicalMetadata` remains the simple item-facing API shape, while SQLite graph
tables power relationship queries and search projection.

Search stays behind `nako-search::SearchIndex`. The first SQLite implementation
uses a persisted `search_documents` table as a fallback. FTS, Tantivy, or
Meilisearch can replace the adapter later without changing catalog writes.

## Consequences

- Actor/director/tag/genre/image queries no longer require scanning item JSON.
- Search documents can be rebuilt from SQLite without rescanning storage.
- Metadata providers and NFO import can converge on one graph upsert path later.
- The first search ranking is intentionally basic; richer tokenization and
  CJK/pinyin/romaji handling remain adapter work.

## Alternatives Considered

- Keep all relationships embedded in `CanonicalMetadata`: rejected because it
  blocks efficient graph pages and search filters.
- Require Meilisearch for MVP: rejected because self-hosted installs should not
  require a separate service.

## Related Workstreams

- Server Foundation Phase 3.6
- Server Foundation Phase 4.0
