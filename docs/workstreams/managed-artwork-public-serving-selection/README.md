# Managed Artwork Public Serving Selection

Status: Active
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

- Define a Public Client image reference shape backed by Taru-owned routes, not
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

## First Executable Task

Continue with MAPS-030: implement explicit Admin publication from a stored
Managed Artwork Artifact to a Selected Artwork record. MAPS-020 has frozen the
public image reference and Selected Artwork model.
