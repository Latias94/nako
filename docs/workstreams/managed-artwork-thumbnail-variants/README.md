# Managed Artwork Thumbnail Variants

Status: Completed
Last updated: 2026-05-19

## Purpose

This lane follows Selected Artwork publication and artifact lifecycle work.
Taru can now publish and serve original Selected Artwork bytes. The next product
boundary is safe image variants: clients need bounded thumbnails and resized
images without learning internal artifact storage, raw source URLs, cache
handles, filesystem paths, or content hashes.

## Goals

- Define the Selected Artwork image variant contract around public image IDs.
- Support explicit bounded `width` and/or `height` query parameters on public
  image byte routes.
- Generate resized variants on demand without persisting cache state in this
  first slice.
- Preserve original image serving when no variant is requested.
- Use redacted, opaque validators that do not expose artifact content hashes.
- Keep gallery management, durable retry/cancel, and missing-artifact repair as
  independent follow-ons.

## Non-Goals

- Persisted thumbnail cache tables or eviction policy.
- Admin gallery/candidate management.
- Durable ingest retry, requeue, cancellation, or runtime controls.
- Missing-artifact repair or re-ingest.
- CDN/provider URL normalization.
- Returning storage URIs, local paths, raw source URLs, cache URIs, or content
  hashes in public or Admin responses.

## Authoritative Docs

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`

## Current Slice

`MATV-020` implemented the first public/Admin contract: `GET/HEAD
/images/{image_id}?width=...&height=...` with bounded on-demand resizing,
redacted headers, OpenAPI/HTTP docs, and focused tests.

## Closeout

The lane is complete. Selected Artwork originals remain available through
`GET/HEAD /images/{image_id}`. Bounded variants are requested with optional
`width` and `height` query parameters, derived on demand, and returned with
opaque presentation ETags that do not expose artifact content hashes. Persisted
variant cache/eviction, gallery management, durable retry/cancel, and
missing-artifact repair remain split follow-ons.
