# Managed Artwork Gallery Candidate Management

Status: Active
Last updated: 2026-05-19

## Purpose

Taru can now accept Artwork Candidates, ingest and store Managed Artwork
Artifacts, publish Selected Artwork, serve original bytes, and derive bounded
public image variants. The next product boundary is management: operators need a
redacted way to inspect candidates, artifacts, and selected slots for one Media
Item, then intentionally change the Selected Artwork choice without learning
raw provider URLs, storage handles, local paths, cache handles, or artifact
content hashes.

## Goals

- Define a redacted Admin gallery read model for one item's artwork candidates,
  stored artifacts, and current Selected Artwork slots.
- Provide explicit Admin commands for selecting, replacing, and eventually
  unpublishing Selected Artwork.
- Keep Public Client image references first-party and selected-artwork based.
- Preserve the existing `/images/{image_id}` and variant query contract.
- Keep raw candidate source data and Managed Artwork Artifact storage authority
  internal.

## Non-Goals

- Persisted thumbnail/variant cache and eviction.
- Durable ingest retry, requeue, cancellation, or runtime controls.
- Missing-artifact repair or re-ingest.
- Provider search, scraping, or automatic artwork ranking.
- Public Client candidate/gallery browsing before the Admin management boundary
  is stable.
- Returning `storage_uri`, `managed-artwork://...`, local paths, raw source
  URLs, `source_uri`, `cache_uri`, provider query strings, addon tokens, file
  contents, or artifact content hashes.

## Authoritative Docs

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`

## Current Slice

`MAGC-020` should implement the first redacted Admin gallery read model:
`GET /admin/v1/items/{item_id}/artwork` or an equivalent Admin route that lets
operators compare current Selected Artwork, accepted/stored artifacts, and
proposed candidates without exposing internal locators.
