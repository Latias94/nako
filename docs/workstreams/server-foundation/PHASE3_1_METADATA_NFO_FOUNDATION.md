# Phase 3.1: Metadata Model and NFO Policy Foundation

## Status

Implemented in the current workspace. This phase defines the metadata and NFO
foundation before real TMDB, Douban, Bangumi, or addon provider calls are added.

## Scope

Phase 3.1 adds:

- richer canonical metadata fields for movie and series foundation work
- image, rating, genre, and credit model primitives
- field-level metadata locks
- provider raw response cache
- explicit metadata merge policy
- minimal movie NFO import/export codec
- ADRs for metadata authority and NFO boundaries

## Metadata Model

`CanonicalMetadata` now includes:

- title and original title
- sort title
- overview
- release date
- runtime minutes
- tagline
- genres
- ratings
- image references
- credits
- external IDs

The model is still intentionally small. It is broad enough to support provider
matching, movie NFO round trips, and later UI/API work without prematurely
modeling every provider-specific field.

## Merge Policy

Metadata refresh is governed by field locks:

- locked fields keep the local value
- unlocked fields may be filled or replaced by incoming provider/NFO metadata
- raw provider responses are cached separately from canonical metadata
- NFO can be treated as local authority when imported with lock policy

The initial merge policy lives in `nako-metadata` and is tested without network
providers.

## NFO Policy

`nako-nfo` contains a minimal movie NFO codec. It supports core tags:

```text
title
originaltitle
sorttitle
plot
releasedate
runtime
tagline
genre
```

Soft-link and hard-link management is not part of the NFO codec. Link behavior
belongs to the VFS/storage layer because backend capabilities vary.

## Persistence

SQLite now persists:

- full metadata JSON on media items
- field locks in `metadata_field_locks`
- raw provider responses in `provider_raw_responses`

Existing scalar columns remain for simple query and compatibility needs.

## Verification

Automated gates:

```powershell
cargo fmt --all -- --check
cargo check --workspace
cargo nextest run --workspace
```

Key test coverage:

- rich metadata round-trip through SQLite
- field lock round-trip
- raw provider cache round-trip
- metadata merge does not overwrite locked fields
- movie NFO core fields round-trip

## Out of Scope

- real TMDB, Douban, or Bangumi HTTP providers
- provider credential storage
- full NFO compatibility with Jellyfin, Kodi, Plex, or Emby variants
- NFO file discovery and filesystem link management
- metadata refresh jobs and API routes
