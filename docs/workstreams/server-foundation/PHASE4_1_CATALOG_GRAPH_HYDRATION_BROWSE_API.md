# Phase 4.1: Catalog Graph Hydration and Browse API

## Goal

Connect the catalog graph foundation to real metadata update paths. Metadata
refresh and NFO import now hydrate normalized graph rows, rebuild search
projection, and expose browse routes for future web or Flutter clients.

This phase still avoids playback, image proxy/cache workers, and full UI work.
The objective is to make catalog data queryable and client-ready.

## Implemented Shape

### Shared Catalog Hydration

`nako-catalog` owns the reusable hydration path:

- load a `MediaItem`;
- clear the item graph edges that are derived from canonical metadata;
- reuse people, genres, tags, collections, and studios by external ID or
  stable natural key;
- upsert item credits, genres, tags, collection memberships, studio links, and
  item images;
- rebuild the item search projection.

This keeps metadata refresh, NFO import, future user edits, and future bulk
repair jobs on the same write path.

### Metadata and NFO Inputs

TMDB movie metadata now carries collection and production-company references
into canonical metadata, in addition to existing title, ratings, images,
credits, genres, and external IDs.

Movie NFO import now maps:

- `<genre>` to provider/local genres;
- `<tag>` to tags;
- `<actor>` to actor credits;
- `<director>` and `<writer>` to crew credits;
- `<poster>`, `<thumb>`, and `<fanart>` to local image references.

NFO local authority still uses field locks according to the configured local
metadata policy.

### Browse API

The server now exposes:

```text
GET /items/{item_id}
GET /items/{item_id}/credits
GET /items/{item_id}/images
GET /people
GET /people/{person_id}
GET /people/{person_id}/items
GET /tags
GET /tags/{tag_id}/items
GET /genres
GET /genres/{genre_id}/items
```

List routes use the existing offset pagination envelope.

## Non-Goals

- No image proxy/cache endpoint yet.
- No thumbnail or preview-frame worker yet.
- No playback, remux, transcode, or HLS path.
- No Douban or Bangumi provider implementation yet.
- No frontend client yet.

## Validation

Coverage added or updated for:

- catalog hydration into people, credits, genres, tags, images, and search;
- metadata refresh graph hydration and search projection;
- NFO import graph hydration and search projection;
- browse HTTP routes for item detail, credits, images, people, tags, and
  genres.

Required gates:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo nextest run --workspace
```
