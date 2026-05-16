# Phase 27.0: Metadata-Catalog Domain Baseline

## Status

Proposed next goal.

## Objective

Create an implementation-ready domain baseline for Taru's metadata-catalog
model before adding more provider breadth or client-facing catalog behavior.

The current server can manage a movie-focused library, local metadata, TMDB
movie refresh, NFO import/export jobs, catalog graph browsing, and search
projection. The next risk is not a missing provider call; it is letting series,
anime, duplicate sources, NFO round trips, artwork, and addon contributions
grow through ad hoc fields.

## Domain Language

Use the terms defined in `CONTEXT.md`:

- **Media Library** is the configured collection boundary for storage,
  metadata, and permission context.
- **Media Source** is one discoverable playable file or remote object.
- **Source Locator** is library-scoped and must not become global identity.
- **Source Fingerprint** is evidence, not identity.
- **Source Duplicate Relationship** links likely duplicate sources without
  automatically merging them.
- **Media Item** is the user-facing catalog entry.
- **Canonical Metadata** is the authoritative browsing/search/playback/export
  metadata.
- **Metadata Source Priority** decides how local, NFO, provider, and future
  addon data fill fields.
- **NFO Round Trip** preserves content Taru does not own.
- **Managed Artwork**, **Artwork Candidate**, and **Selected Artwork** are
  distinct concepts.

## Design Questions

M27.0 should answer these before implementation:

- Is `MediaKind` enough for movies, series, seasons, episodes, collections,
  and extras, or does it need a larger hierarchy?
- Does the existing `MediaItem.item_id` on `MediaSource` still work for
  multi-source items, alternate editions, and duplicate relationships?
- Which duplicate-source evidence is strong enough to create a **Source
  Duplicate Relationship**?
- Which fields are canonical, which are provider-specific, and which are local
  locks?
- How do TMDB series, Douban, and Bangumi map into ordinary Taru item kinds
  without hard-coding anime as a separate media kind?
- What must NFO export preserve to satisfy **NFO Round Trip**?
- What should clients receive for artwork: provider URLs, managed image IDs,
  variant URLs, or selected artwork references?
- Which decisions need an ADR before schema work starts?

## Proposed Implementation Sequence

1. Audit current code and docs against `CONTEXT.md`.
2. Write or update ADRs for item hierarchy, source relationships, metadata
   authority, and artwork ownership.
3. Move active metadata/catalog TODOs out of `server-foundation`.
4. Draft the schema/repository change plan for M27.1.
5. Only then implement provider breadth in M27.2.

## Non-Goals

- No provider API implementation in M27.0.
- No client UI.
- No automatic duplicate merge.
- No destructive NFO rewrite.
- No addon direct database or filesystem write path.

## Validation

Close-out gate for the planning slice:

- `git diff --check`
