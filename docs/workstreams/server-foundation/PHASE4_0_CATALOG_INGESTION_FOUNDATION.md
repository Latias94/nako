# Phase 4.0: Catalog Ingestion Foundation

## Goal

Turn the earlier catalog graph design into the first server-side library loop:
configure a library, scan a local VFS root, persist media items and source
state, write normalized catalog graph records, project items into search, and
persist artwork/preview work as bounded background tasks.

This phase intentionally stays below provider-specific scraping and playback.
It creates the database and service foundation those features need.

## Implemented Shape

### Catalog Graph

`nako-core` now models first-class catalog entities:

- `Person` and `ItemCredit`
- `Genre` and `ItemGenre`
- `Tag` and `ItemTag`
- `Collection` and `CollectionItem`
- `Studio` and `ItemStudio`
- `ImageAsset`

`nako-db` persists them through migration `0007_catalog_ingestion.sql` and
implements `CatalogRepository` for SQLite. Existing `CanonicalMetadata` remains
the item-facing shape; the graph is stored alongside it rather than replacing
the JSON metadata payload.

### Scan State

Scans now create durable `scan_snapshots`, `directory_snapshots`, and
`source_states` records. `LibraryIndexService` records the scan ID, directory
metadata, source fingerprint metadata, and tombstones sources that disappear
from a later scan.

The local VFS backend now exposes a lightweight local fingerprint derived from
file size and modified timestamp. This avoids content-hashing large libraries
while still supporting basic incremental detection.

### Search Projection

`nako-search` remains the adapter boundary. SQLite implements `SearchIndex`
using a persisted `search_documents` projection table. The first implementation
uses simple SQLite-backed text matching instead of requiring FTS5 or
Meilisearch, because external or feature-gated search engines should remain
replaceable.

`LibraryIndexService` rebuilds an item search document after scan ingestion.
The HTTP server exposes:

```text
GET /search?q=matrix&facet=genre:sci-fi
```

### Artwork and Preview Task Foundation

`ImageAsset` records image ownership, image kind, provider provenance, cache
URI, dimensions, selection state, content hash, and etag.

`artwork_tasks` persists queued work for:

- `artwork.fetch`
- `artwork.resize`
- `artwork.preview`
- `artwork.cleanup`

Tasks carry status, retry attempts, max attempts, and resource class. Core
queue options define conservative per-resource concurrency defaults. Actual
download, resize, and ffmpeg preview execution remain future work.

## Non-Goals

- No TMDB/Douban/Bangumi graph upsert path yet.
- No full thumbnail generation worker yet.
- No transcoding, playback, remuxing, or HLS path.
- No plugin runtime.
- No local AI model, vector index, or recommendation engine.
- No real remote drive backend yet; VFS boundaries remain prepared for it.

## Validation

The phase is covered by tests for:

- catalog graph round-trips in SQLite;
- scan snapshots, directory snapshots, source state, and tombstone behavior;
- local VFS scan ingestion into media items and search projection;
- SQLite search route through HTTP;
- image asset and artwork task persistence;
- existing bounded probe concurrency behavior.

Required gates:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo nextest run --workspace
```
