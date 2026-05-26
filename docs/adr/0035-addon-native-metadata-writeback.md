# 0035. Addon Native Metadata Writeback

Date: 2026-05-26

## Status

Accepted

## Context

Addon metadata writeback originally accepted a narrow patch with scalar fields,
genres, and tags. That shape was enough for simple provider suggestions, but it
could not persist richer catalog graph data such as people, credits, studios,
collections, external IDs, ratings, and image references.

The official metadata scraper now produces MDCx-style AV facts. Keeping those
facts response-only would make AV provider expansion incomplete: the sidecar
could scrape actors, directors, studios, series, thumbnails, and fanart, while
Nako would still persist only the narrow subset.

## Decision

Nako addon `metadata_write` now accepts a canonical metadata-shaped patch. The
protocol crate remains independent from `nako-core`, but the wire shape mirrors
canonical metadata fields:

- scalar title and descriptive fields;
- genres and tags;
- ratings;
- image references;
- credits;
- collections;
- studios;
- external IDs.

The server maps this protocol payload into `CanonicalMetadata`, merges it with
local locks, and runs full catalog graph projection after apply. We intentionally
do not keep a compatibility shim for the older narrow write path; the narrow
fields remain valid only because they are a natural subset of the canonical
metadata shape.

## Consequences

- Addons can submit complete metadata graph facts through one side effect.
- Catalog graph hydration stays centralized in `nako-catalog`.
- The metadata write adapter is simpler because it does not own partial graph
  update rules.
- Old addon payloads that relied on unsupported graph fields being rejected are
  no longer meaningful; the protocol is considered broken for this phase.
