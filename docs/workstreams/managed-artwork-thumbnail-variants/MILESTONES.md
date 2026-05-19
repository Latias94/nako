# Managed Artwork Thumbnail Variants Milestones

Status: Completed
Last updated: 2026-05-19

## M0 - Open Variant Contract

Exit criteria:

- Workstream docs exist.
- Public route shape and non-goals are explicit.
- Gallery, retry/cancel, missing repair, and persisted cache are split.

Status: Done.

## M1 - On-Demand Variants

Exit criteria:

- `GET /images/{image_id}` remains compatible for originals.
- `GET /images/{image_id}?width=...&height=...` returns bounded variants.
- `HEAD` returns matching variant presentation headers without a body.
- Image ETags no longer expose artifact content hash values.
- OpenAPI and HTTP docs include the variant query parameters.

Status: Done.

## M2 - Closeout

Exit criteria:

- Focused tests and relevant workspace checks pass.
- Workstream docs record evidence and follow-ons.
- No persisted cache, gallery, retry/cancel, or repair behavior is hidden in
  this lane.

Status: Done.

Closeout notes:

- Variant serving is on-demand only; no persisted variant files or DB rows were
  added.
- Public/Admin DTOs remain redacted and do not include storage locators or
  artifact content hashes.
- Persisted variant cache/eviction, gallery/candidate management,
  retry/cancel, and repair/re-ingest are still independent follow-ons.
