# Managed Artwork Gallery Candidate Management Handoff

Status: Active
Last updated: 2026-05-19

## Current State

This lane is open and `MAGC-010` is complete.

The lane owns Admin management for item-scoped artwork choices after the
candidate ingest, artifact storage, selected artwork publication, public image
serving, lifecycle cleanup, remediation policy, and thumbnail variant lanes.

## Next Step

Implement `MAGC-020`:

- add an item-scoped Admin artwork gallery read model;
- include redacted Artwork Candidate summaries;
- include redacted Managed Artwork Artifact summaries;
- include current Selected Artwork public image refs;
- avoid raw source URLs, `source_uri`, `cache_uri`, storage URIs,
  `managed-artwork://...`, local paths, artifact content hashes, and token
  material;
- document the route in `docs/api/HTTP_API.md`;
- add focused API/server/db tests with a `managed_artwork_gallery` filter.

## Blockers

None known.

## Follow-Ons Outside This Lane

- Public Client candidate/gallery browsing.
- Persisted thumbnail/variant cache and eviction.
- `managed-artwork-ingest-runtime-controls`.
- Missing-artifact repair/re-ingest.
- Provider search, scraping, or automatic artwork ranking.

Keep the redaction invariant in all follow-ons: no `storage_uri`, source URL,
`source_uri`, `cache_uri`, local path, `managed-artwork://...`, artifact content
hash, file contents, or addon/provider token material in Public Client/Admin
DTOs.
