# Managed Artwork Fetch Artifact Storage

Status: Completed
Last updated: 2026-05-19

This workstream follows `managed-artwork-ingest-selection`. The previous lane
proved that Admin candidate acceptance can create internal
`managed_artwork_ingests` state and a durable `managed_artwork_ingest` job
without publishing public artwork. This lane owns the next private boundary:
fetch the accepted candidate source through Taru-controlled policy, validate
the image, and store artifact bytes as internal managed artwork.

Authoritative docs:

- `DESIGN.md`
- `MILESTONES.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `HANDOFF.md`
- `../managed-artwork-ingest-selection/`
- `../../adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `../../adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`

## Goals

- Consume queued `managed_artwork_ingest` jobs through a Taru-owned worker or
  equivalent first-party runtime boundary.
- Fetch only internally accepted candidate sources with explicit resource
  budgets, timeout, retry, and cancellation semantics.
- Validate content type, byte size, dimensions, and image decodability before
  persisting bytes as managed artwork.
- Write internal artifact bytes and `managed_artwork_artifacts` records without
  creating public `ImageAsset` rows or selected artwork.
- Report failures with safe codes and redacted job summaries.

## Non-Goals

- No Public Client image-serving route in this lane.
- No selected artwork publication.
- No thumbnail or resize pipeline unless it is needed only for validation.
- No Addon Side Effect fetch/cache behavior.
- No artwork sidecar file export; that remains Library File Write scope.

## Closeout Outcome

- Accepted managed artwork ingest jobs can be processed by a Taru-owned Admin
  runtime seam into internal artifact bytes and `managed_artwork_artifacts`
  metadata.
- Fetch, validation, local internal artifact storage, claim/commit/failure
  transitions, and redacted Admin responses are in place.
- Public image serving, thumbnails, selected artwork publication, durable
  retry/requeue, cancellation, and orphan cleanup remain explicit follow-ons.
