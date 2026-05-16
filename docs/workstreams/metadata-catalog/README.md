# Metadata-Catalog Workstream

## Purpose

This workstream owns Taru's video-first media-library domain model after the
first movie-focused MVP. It covers **Media Item** hierarchy, **Media Source**
relationships, **Provider Mapping**, **Canonical Metadata**, **Media Technical
Facts**, **Library Item State**, **User Playback State**, provider breadth,
NFO round trips, artwork management, and catalog/search expansion.

The project language is defined in `CONTEXT.md`; use those terms in this
workstream instead of drifting back to file-centric or provider-centric names.

## Status

M27.0 design baseline completed. M27.1 schema and repository slice completed.
M27.2 local inference and provisional hierarchy completed. M27.3 provider and
NFO expansion is planned next.

## Top-Level Tracking

- [Goal map](../../GOALS.md)
- [Roadmap](../../ROADMAP.md)
- [Milestones](MILESTONES.md)
- [TODO](TODO.md)
- [Phase 27.0 metadata-catalog domain baseline](PHASE27_0_METADATA_CATALOG_DOMAIN_BASELINE.md)
- [Phase 27.1 catalog schema and repository slice](PHASE27_1_CATALOG_SCHEMA_REPOSITORY_SLICE.md)
- [Phase 27.2 local inference and provisional hierarchy](PHASE27_2_LOCAL_INFERENCE_PROVISIONAL_HIERARCHY.md)

## Goals

- Model **Media Library** as the configured collection boundary for storage,
  metadata policy, permission context, and presentation defaults.
- Keep Taru video-first in implementation while preserving a broader
  media-server model through **Media Domain** and **Library Preset**.
- Keep **Media Source** identity separate from **Media Item** identity.
- Support **Source Duplicate Relationship** without automatically merging
  sources or collapsing library context.
- Expand **Media Item** beyond movies to series, seasons, episodes,
  **Episode-Like Item**, **Extra Item**, **Franchise Collection**, and unknown
  video items.
- Map provider-specific concepts through **Provider Subject** and **Provider
  Mapping** instead of replacing Taru item identity.
- Make **Canonical Metadata** resolution explicit through **Metadata Source
  Priority**, local edits, NFO, built-in providers, and future addons.
- Keep **Canonical Metadata**, **Media Technical Facts**, **Library Item
  State**, and **User Playback State** separate.
- Distinguish **Genre** from **Tag**, and **Review Rating**, **Content
  Rating**, and **User Rating** from each other.
- Treat **Browse Facet** and **Sort Key** as explicit public client contracts,
  not raw database-column exposure.
- Preserve local files through **NFO Round Trip** and mediated **Library File
  Write** behavior.
- Serve client-facing artwork through **Managed Artwork**, **Artwork
  Candidates**, and **Selected Artwork** rather than provider hotlinks.
- Keep search indexed from the catalog graph rather than raw provider payloads.

## Non-Goals

- No client UI implementation.
- No full music, podcast, photo, document, or online-catalog implementation in
  M27.
- No provider-specific implementation before the domain model is stable.
- No destructive NFO rewrite.
- No automatic duplicate-source merge without a separate high-confidence rule.
- No addon write path that bypasses Taru-owned APIs and permission checks.
- No raw database columns as client filtering or sorting contracts.

## Boundary Rules

- `taru-core` owns durable domain vocabulary, IDs, records, and repository
  traits.
- `taru-db` owns schema, migrations, and repository adapters.
- `taru-metadata` owns built-in provider adapters and provider payload mapping.
- `taru-nfo` owns local NFO parsing/export behavior and round-trip policy.
- `taru-vfs` owns backend write/link capability reporting consumed by
  **Library File Write** workflows.
- `taru-catalog` owns graph hydration and search projection updates.
- `taru-server::app` owns orchestration and policy composition.
- `taru-api` owns explicit client-facing DTOs.
- Client browse routes expose named **Browse Facets** and **Sort Keys** only
  after they are intentionally supported.

## Related Workstreams

- [ADR 0021](../../adr/0021-video-first-media-server-domain-model.md): the
  video-first media-server domain decision.
- [server-foundation](../server-foundation/README.md): historical metadata,
  NFO, catalog, artwork, and search backlog.
- [metadata-operations](../metadata-operations/README.md): completed metadata
  maintenance, diagnostics, and provider runtime hardening.
- [addons-automation](../addons-automation/README.md): addon and automation
  boundaries that may later contribute metadata suggestions or managed
  artifacts.
