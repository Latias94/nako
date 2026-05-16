# Metadata-Catalog Workstream

## Purpose

This workstream owns Taru's media-library domain model after the first
movie-focused MVP. It covers **Media Item** hierarchy, **Media Source**
relationships, **Canonical Metadata**, provider breadth, NFO round trips,
artwork management, and catalog/search expansion.

The project language is defined in `CONTEXT.md`; use those terms in this
workstream instead of drifting back to file-centric or provider-centric names.

## Status

Proposed for M27.

## Top-Level Tracking

- [Goal map](../../GOALS.md)
- [Roadmap](../../ROADMAP.md)
- [Milestones](MILESTONES.md)
- [TODO](TODO.md)
- [Phase 27.0 metadata-catalog domain baseline](PHASE27_0_METADATA_CATALOG_DOMAIN_BASELINE.md)

## Goals

- Model **Media Library** as the configured collection boundary for storage,
  metadata policy, and permission context.
- Keep **Media Source** identity separate from **Media Item** identity.
- Support **Source Duplicate Relationship** without automatically merging
  sources or collapsing library context.
- Expand **Media Item** beyond movies to series, seasons, episodes,
  collections, and future extras.
- Make **Canonical Metadata** resolution explicit through **Metadata Source
  Priority**, local edits, NFO, built-in providers, and future addons.
- Preserve local files through **NFO Round Trip** and mediated **Library File
  Write** behavior.
- Serve client-facing artwork through **Managed Artwork**, **Artwork
  Candidates**, and **Selected Artwork** rather than provider hotlinks.
- Keep search indexed from the catalog graph rather than raw provider payloads.

## Non-Goals

- No client UI implementation.
- No provider-specific implementation before the domain model is stable.
- No destructive NFO rewrite.
- No automatic duplicate-source merge without a separate high-confidence rule.
- No addon write path that bypasses Taru-owned APIs and permission checks.

## Boundary Rules

- `taru-core` owns durable domain vocabulary, IDs, records, and repository
  traits.
- `taru-db` owns schema, migrations, and repository adapters.
- `taru-metadata` owns built-in provider adapters and provider payload mapping.
- `taru-nfo` owns local NFO parsing/export behavior and round-trip policy.
- `taru-catalog` owns graph hydration and search projection updates.
- `taru-server::app` owns orchestration and policy composition.
- `taru-api` owns explicit client-facing DTOs.

## Related Workstreams

- [server-foundation](../server-foundation/README.md): historical metadata,
  NFO, catalog, artwork, and search backlog.
- [metadata-operations](../metadata-operations/README.md): completed metadata
  maintenance, diagnostics, and provider runtime hardening.
- [addons-automation](../addons-automation/README.md): addon and automation
  boundaries that may later contribute metadata suggestions or managed
  artifacts.
