# Managed Artwork Public Serving Selection

Status: Completed
Last updated: 2026-05-19

This workstream follows `managed-artwork-fetch-artifact-storage`. The previous
lane proved that accepted managed artwork ingest jobs can fetch, validate, and
store internal artifact bytes with opaque `managed-artwork://...` storage
authority. This lane owns the next public boundary: publish a stored Managed
Artwork Artifact as the current Selected Artwork for an item, expose only
first-party Public Client image references, and serve bytes without leaking
storage, source, cache, or filesystem details.

Authoritative docs:

- `DESIGN.md`
- `MILESTONES.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `HANDOFF.md`
- `../managed-artwork-fetch-artifact-storage/`
- `../../adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `../../adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`

## Goals

- Define a Public Client image reference shape backed by Nako-owned routes, not
  provider URLs, local paths, cache handles, or `managed-artwork://...` storage
  URIs.
- Introduce Selected Artwork publication state that maps an item and image kind
  to a stored Managed Artwork Artifact.
- Add an explicit Admin publication command from a stored artifact to Selected
  Artwork without giving Addons direct publication or filesystem authority.
- Serve selected image bytes through Public Client routes using internal
  artifact storage only behind the server boundary.
- Remove or replace public DTO fields that expose `source_uri`, `cache_uri`,
  `storage_uri`, local paths, raw source URLs, or provider image URLs.

## Non-Goals

- No thumbnail or resize generation in this lane.
- No durable retry/requeue, cancellation API, or orphan artifact cleanup.
- No Artwork Export to sidecar files.
- No Addon-side fetch, cache, selection, or publication behavior.
- No public artwork gallery or candidate-management UI; this lane publishes the
  current Selected Artwork only.

## Closeout

This lane is complete. Stored Managed Artwork Artifacts can be explicitly
published as Selected Artwork, Public Client item image responses use
`PublicImageRefDto`, and selected image bytes are served through
`GET/HEAD /images/{image_id}` without exposing source, cache, storage, or local
filesystem locators.

## Follow-On Splits

- `managed-artwork-thumbnail-variants`: thumbnail/resize generation,
  responsive variants, cache validators, and range/variant serving policy.
- `managed-artwork-ingest-runtime-controls`: durable retry/requeue,
  cancellation, and Admin/runtime controls for managed artwork ingest jobs.
- `managed-artwork-artifact-lifecycle-cleanup`: orphan artifact detection,
  selected-artwork retention protection, artifact garbage collection, and
  operator diagnostics.
- `managed-artwork-gallery-candidate-management`: public/Admin browsing for
  candidates and artwork galleries after the Selected Artwork boundary is
  stable.
