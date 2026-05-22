# Phase 3.6: Catalog Graph, Artwork Cache, Search, and Scan Strategy

## Goal

Define the foundation that prevents Nako's catalog, UI, and scanner from being
limited by an item-only metadata model. This phase is design-only: it records
the target model and implementation sequence for people, tags, artwork,
search, and incremental scanning before adding more provider-specific metadata.

## Why This Phase Exists

Movie-server features such as actor pages, director pages, tag filters,
collection pages, search suggestions, poster grids, and incremental rescans all
depend on shared catalog infrastructure. If Nako keeps these as ad hoc JSON
fields on `MediaItem`, future TMDB series support, Bangumi, Douban, NFO actors,
and clients will need repeated migrations.

The design target is:

- canonical metadata remains the simple item-facing API shape;
- normalized catalog graph tables power relationships, filtering, and search;
- image assets are cached and served through an explicit artwork pipeline;
- scans become incremental, resumable, and safe for remote storage.

## Catalog Graph

### Core Entities

The catalog graph should be normalized around these entities:

- `MediaItem`: movie, series, season, episode, collection, extra, and future
  audio/photo item kinds.
- `Person`: actor, director, writer, producer, creator, and other contributors.
- `ItemCredit`: join table between `MediaItem` and `Person`.
- `Genre`: provider or local genre vocabulary.
- `Tag`: user/local classification that should not be overwritten by providers.
- `Collection`: provider/local grouping such as movie collections, playlists,
  franchises, or user-defined sets.
- `Studio`: production company, broadcaster, label, or publisher.
- `ImageAsset`: poster, backdrop, logo, thumbnail, banner, person image, and
  generated preview frames.
- `ExternalId`: provider IDs for items, people, collections, studios, and
  image assets.

### Relationship Rules

- People are first-class records. Credits point to people and items.
- Credit roles are normalized but extensible; unknown provider roles are stored
  as `Other`.
- Provider genres and user tags are separate concepts.
- Collections are explicit entities, not only tags.
- The same person can have multiple provider IDs.
- The same image can be attached to an item, person, collection, or studio.
- User-created tags and collections are local authority by default.

### Query Patterns To Support

The model should support these without scanning all JSON metadata:

- item detail page: item, sources, images, tags, genres, credits;
- person page: person metadata plus related items grouped by role;
- director page: same as person page with role filter;
- tag page: items with tag plus pagination;
- genre page: items with genre plus pagination;
- collection page: ordered items;
- duplicate/match audit: items grouped by external ID or fingerprint;
- recommendations later: graph neighborhoods and provider suggestions.

## NFO and Provider Mapping

NFO actor/director support should map into the same graph:

- `<actor>` entries create or match `Person` records.
- actor roles create `ItemCredit` rows with role `actor`.
- director/writer/producer tags create corresponding credit rows.
- NFO-imported people may write `MetadataSource::Nfo` field locks when local
  authority policy requires it.

Provider metadata should follow the same path:

- TMDB person IDs map to `Person` external IDs.
- Bangumi and Douban person IDs map to provider-specific external IDs.
- Provider images are cached as `ImageAsset` candidates, not directly embedded
  into every query response.

## Artwork Cache and Preview Performance

### Image Asset Model

`ImageAsset` should store:

- owner kind and owner ID, such as item/person/collection/studio;
- image kind, such as poster/backdrop/logo/thumbnail/banner/person/preview;
- source URI and provider;
- local cache URI or storage locator;
- width, height, language, score, and selected/default flag;
- content hash, etag, and fetched/generated timestamp;
- variant metadata for original, poster grid, detail page, thumbnail, and blur
  placeholder sizes.

### Runtime Rules

- API list routes should return image references, not inline image bytes.
- Clients should lazy-load images and request only the sizes they need.
- The server should expose image URLs with cache headers and etags.
- Image downloads and thumbnail generation must use bounded concurrency.
- Remote image fetches must be rate-limited by provider.
- Preview-frame extraction from video should be queued work, not synchronous
  list-page work.
- Failed image fetches should be cached briefly to avoid retry storms.

### Suggested Resource Classes

- `artwork.fetch`: remote poster/backdrop/person image downloads.
- `artwork.resize`: CPU image resize and format conversion.
- `artwork.preview`: ffmpeg frame extraction.
- `artwork.cleanup`: stale cache eviction.

Each class should have independent concurrency limits because network image
downloads, CPU image resizing, and ffmpeg preview extraction stress different
resources.

## Search Strategy

### Search Boundary

Search should stay behind an internal `nako-search` trait:

- embedded implementation: SQLite FTS for the first production MVP;
- richer embedded implementation: Tantivy when ranking/faceting needs grow;
- optional external adapter: Meilisearch for users who want a dedicated search
  service.

Search should not become a hard runtime dependency for the server MVP.

### Indexed Documents

The search index should contain denormalized documents derived from the catalog
graph:

- item title, original title, sort title, aliases;
- overview and tagline;
- file name and parsed name;
- genres, user tags, collections, studios;
- people names grouped by role;
- external IDs;
- media kind, domain, library ID, release year, runtime, watched state later.

### Index Updates

- Library scan, metadata refresh, NFO import/export, and future user edits
  should emit index-update events.
- Indexing should be idempotent by item ID and version.
- Rebuild should be possible from SQLite without re-scanning storage.
- Search updates should be eventually consistent; catalog writes should not
  block on optional external search services.

### Query Features

Initial search should support:

- prefix and fuzzy title search;
- person/tag/genre filters;
- library filter;
- item-kind filter;
- pagination;
- stable sorting by relevance, title, release date, and recently added.

Chinese/Japanese/Korean search can start with normalized text and later add
tokenizers or pinyin/romaji support behind the search adapter.

## Scan Strategy

### Scan State

Scanning should move from "list and upsert" toward a durable snapshot model:

- `scan_snapshot`: one record per scan run;
- `directory_snapshot`: directory URI, etag, modified timestamp, child count;
- `source_state`: media source URI, size, modified timestamp, fingerprint,
  last seen scan ID, and tombstone flag;
- `path_alias`: old URI to new URI when rename/move detection is confident.

The first implementation can keep this simple, but the schema should leave
room for remote stores with expensive listing and weak modification times.

### Incremental Scan Rules

- Use VFS `etag`, `fingerprint`, size, and modified timestamp when available.
- Skip unchanged directories when a backend can prove they are unchanged.
- For local files, use watcher events only as hints; rescan remains source of
  truth.
- Debounce bursty watcher events.
- Track tombstones for missing sources instead of deleting immediately.
- Detect rename/move by fingerprint when possible.
- Keep scans idempotent so repeated jobs are safe.
- Isolate per-source failures and record them in job summaries.

### Remote Storage Rules

- Remote backends declare `EXPENSIVE_LISTING`, `RATE_LIMITED`, and
  `REMOTE_LATENCY` capabilities.
- Scanner concurrency must be lower for remote backends.
- Directory listing cache should use backend etag/fingerprint when available.
- Byte-range cache or local staging should be chosen before probe/transcode.
- Retry policy should distinguish provider rate limit, transient network
  failure, auth failure, and missing object.

## Storage Cache Strategy

### Directory Metadata Cache

The directory cache stores listing and stat metadata, not media bytes:

- directory URI, backend scheme, etag, modified timestamp, and listing cursor;
- child object URI, kind, size, modified timestamp, etag, and fingerprint;
- freshness deadline and backend-specific cache policy;
- last scan ID and last successful listing timestamp;
- failure state for rate limits, auth errors, and transient network errors.

The scanner can use this cache to avoid relisting unchanged remote directories.
Local backends may keep short freshness windows or bypass the cache when watcher
events indicate changes.

### Byte-Range Media Cache

The byte-range cache stores media byte windows for probe, preview, direct play,
and future transcode staging:

- source URI and byte range;
- local cache path or storage locator;
- etag/fingerprint of the source object when the range was cached;
- size, fetched timestamp, last access timestamp, and eviction priority;
- resource class that filled the range, such as probe, preview, direct play, or
  transcode.

Probe and preview jobs should request the smallest ranges they need. Transcode
jobs may choose full local staging when random access is required and the
backend is remote or slow. Direct play should prefer upstream range reads when
the backend is range-readable and low latency, and should fall back to cached
ranges when the backend is remote, expensive, or rate-limited.

Cache invalidation must compare source etag/fingerprint before reuse. Cache
eviction should be size-bounded and independent from metadata database cleanup.

## API Shape

Future API routes should grow around graph resources:

```text
GET /items
GET /items/{item_id}
GET /people
GET /people/{person_id}
GET /people/{person_id}/items
GET /tags
GET /tags/{tag_id}/items
GET /genres
GET /genres/{genre_id}/items
GET /collections
GET /collections/{collection_id}/items
GET /images/{image_id}
GET /search?q=...
```

List routes should always support pagination. Expensive expansions should be
explicit query parameters, for example `include=images,credits`.

## Implementation Sequence

1. Add catalog graph domain models in `nako-core`.
2. Add SQLite tables for people, credits, tags, genres, collections, studios,
   image assets, and search-index bookkeeping.
3. Teach metadata refresh and NFO import to upsert graph records.
4. Add image asset cache records and image proxy routes.
5. Add thumbnail generation jobs with bounded resource classes.
6. Add SQLite FTS search adapter behind `nako-search`.
7. Add incremental scan snapshot tables and scanner summaries.
8. Add directory metadata cache, byte-range media cache, and remote backend
   policies.

## Non-Goals

- Do not implement a recommendation engine in this phase.
- Do not require Meilisearch or another external service.
- Do not add local vector search.
- Do not expose every graph API before the storage model exists.
- Do not parse every NFO variant before the graph model is stable.

## Validation For This Design Phase

This phase is complete when:

- catalog graph entities and relationships are documented;
- artwork cache and preview-generation performance rules are documented;
- search indexing and adapter strategy are documented;
- incremental scan and remote-storage scan rules are documented;
- milestone and TODO documents point to this design.
