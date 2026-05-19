# Managed Artwork Thumbnail Variants Design

Status: Completed
Last updated: 2026-05-19

## Problem

Selected Artwork currently exposes only the original image bytes at
`/images/{image_id}`. That is safe but inefficient for poster grids, detail
headers, mobile clients, and future admin UI. A variant contract must not turn
Managed Artwork Artifact storage into a public API, and it must not leak raw
source/cache/storage values through headers or DTO fields.

## Target State

- Public Clients keep using the Selected Artwork public image ID.
- Variants are requested explicitly with bounded `width` and/or `height` query
  parameters on `GET/HEAD /images/{image_id}`.
- The server preserves aspect ratio and never upscales the source image.
- The first implementation derives bytes on demand and does not persist
  variant files or DB rows.
- `PublicImageRefDto.url` remains the original image URL; clients add query
  parameters for concrete variants.
- Admin publication responses reuse the same redacted image reference contract.
- Public and Admin responses never expose `storage_uri`,
  `managed-artwork://...`, local paths, raw source URLs, `source_uri`,
  `cache_uri`, provider query strings, addon tokens, file contents, or artifact
  content hashes.

## Public Route Contract

```text
GET  /images/{image_id}?width=300
GET  /images/{image_id}?height=300
GET  /images/{image_id}?width=300&height=450
HEAD /images/{image_id}?width=300
```

Rules:

- `width` and `height` are optional positive integers.
- At least one dimension must be present to request a variant.
- Dimensions are capped by the server's artwork image limits.
- When both dimensions are present, the image fits within the bounding box.
- When only one dimension is present, the other dimension is derived from the
  original aspect ratio.
- The server never upscales. If the requested box is larger than the source,
  the original dimensions are returned.
- Unsupported media types, corrupt images, or impossible dimensions return a
  redacted error.
- The first on-demand variant encoder returns `image/png` bytes.

## Validator Policy

Original serving previously used the artifact content hash as the public ETag.
This lane changes image validators to opaque presentation validators derived
from public selected-image identity and the requested variant key. They are
stable enough for cache validation but do not expose artifact content hashes.

## Storage Policy

The first slice is on-demand only:

- no variant DB table;
- no variant files under the artifact root;
- no cleanup or eviction policy;
- no background generation.

Persisted variants can be added later once the public contract and redaction
tests are stable.

## Assumptions

| Assumption | Confidence | Evidence | Mitigation |
| --- | --- | --- | --- |
| Query parameters are the smallest compatible extension to `/images/{image_id}`. | High | Existing public image references already point at `/images/{image_id}`. | Keep original URL unchanged and add optional OpenAPI parameters. |
| On-demand derivation is acceptable for the first slice. | Medium | Artwork images are bounded at ingest by configured max dimensions and bytes. | Keep variant dimensions bounded and split persisted cache later. |
| Opaque presentation ETags are sufficient for clients. | Medium | Current clients only require a cache validator, not artifact content hash semantics. | Keep field name `etag`, but stop using content hash values. |

## Splits

- Persisted variant cache and eviction belong to a later storage/cache lane.
- Gallery/candidate management belongs to
  `managed-artwork-gallery-candidate-management`.
- Durable retry/requeue/cancellation belongs to
  `managed-artwork-ingest-runtime-controls`.
- Missing-artifact repair belongs to a future repair/re-ingest lane.

## Closeout Condition

This lane can close when public image routes support bounded on-demand variants,
original image serving remains compatible, OpenAPI/HTTP docs describe the
contract, and tests prove no storage URI, path, source/cache URI, or content hash
leaks through DTOs or image headers.

Status: closed on 2026-05-19. The closeout condition is met by MATV-020 and
MATV-030 evidence in `EVIDENCE_AND_GATES.md`.
