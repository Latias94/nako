# 0007: Define Metadata Merge Policy and Local Authority

## Status

Proposed

## Context

Taru will ingest metadata from local file names, NFO files, TMDB, Douban,
Bangumi, IMDb, and future addons. These sources can disagree. Users also need
to preserve local edits and local NFO files without provider refreshes silently
overwriting them.

## Decision

Use canonical metadata on `MediaItem` plus explicit field locks and raw provider
response cache.

Authority order:

- user-locked fields are never overwritten by automatic refresh
- NFO/local metadata is treated as local authority when a field is locked or
  imported with local-authority policy
- provider metadata can fill unlocked fields
- provider priority is configurable later; early implementation uses explicit
  merge calls rather than hidden global priority

Field locks are stored per item and per metadata field. Initial lockable fields
include:

- title
- original title
- sort title
- overview
- release date
- runtime
- tagline
- genres
- ratings
- images
- credits
- external IDs

Provider raw responses are cached as JSON. Cache entries are keyed by item,
provider, and provider key. Raw cache is for audit, debugging, and repeatable
merge work; canonical metadata remains the user-facing item state.

## Consequences

- Metadata refresh can be made idempotent and explainable.
- Users can protect local edits before provider integrations exist.
- Raw provider cache provides a stable persistence boundary for later TMDB,
  Douban, and Bangumi adapters.
- Merge policy needs careful tests whenever a new field is added.

## Alternatives Considered

- Provider always overwrites local metadata: simple but hostile to self-hosted
  local-library workflows.
- Keep every provider field separate forever: maximally auditable but too heavy
  for early UI and API work.
- Let NFO fully replace canonical metadata unconditionally: predictable for NFO
  users but dangerous when NFO files are partial.

## Related Workstreams

- `docs/workstreams/server-foundation/`
