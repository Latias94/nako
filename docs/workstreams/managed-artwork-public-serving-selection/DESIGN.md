# Managed Artwork Public Serving Selection Design

Status: Active
Last updated: 2026-05-19

## Problem

`managed-artwork-fetch-artifact-storage` intentionally stopped at internal
artifact storage. Taru can now store validated Managed Artwork bytes, but Public
Client applications still have no safe image reference for those artifacts.
The existing catalog image surface was designed earlier around `ImageAsset` and
still exposes provider/cache-oriented fields:

- `ImageAssetDto.source_uri`
- `ImageAssetDto.cache_uri`
- `ImageAssetDto.selected`
- `ImageRefDto.uri` inside `CanonicalMetadataDto.images`
- OpenAPI schemas and HTTP docs that still describe public `ImageAsset` rows

That model is not acceptable for Managed Artwork. A Public Client image
reference must be a Taru-owned route or opaque identifier, while the
`managed-artwork://...` storage URI, local artifact path, source URL, cache URI,
and Addon/provider provenance stay internal or Admin-redacted.

## Target State

- `ManagedArtworkArtifactRecord` remains the internal byte/storage authority.
- A new Selected Artwork publication record owns the public presentation
  decision for one item and one image kind.
- Public Client DTOs expose first-party image references only, for example a
  stable ID plus a relative route such as `/api/v1/images/{image_id}`.
- Public image serving resolves the public image ID to the currently published
  artifact, checks the item/library boundary, reads bytes through the internal
  artifact storage port, and streams them with safe headers.
- Existing `ImageAsset`/metadata image provenance can remain internal until it
  is either migrated or deleted, but it must no longer be the public API
  authority when it can carry raw source/cache URIs.

## Domain Boundary

### Managed Artwork Artifact

The artifact is the durable stored byte object produced by accepted ingest. It
has an internal storage URI and validation metadata. It is not automatically
public and must not be serialized to Public Client DTOs.

### Selected Artwork

Selected Artwork is the publication mapping:

- owner: item in this lane
- kind: poster, backdrop, logo, banner, thumbnail, or future image kind
- artifact: one stored Managed Artwork Artifact
- public identity: stable route ID for clients

The mapping should be idempotent. Publishing the same artifact for the same
item/kind should return the same Selected Artwork record. Publishing a different
artifact for the same item/kind should update the mapping without exposing the
old artifact as still selected.

### Public Image Reference

The public reference is a client contract, not a storage locator. It may include:

- `id`
- `owner`
- `kind`
- `url`
- `width`
- `height`
- `language`
- `media_type`
- safe cache validators such as an ETag derived from public identity and
  content hash

It must not include:

- `source_uri`
- `cache_uri`
- `storage_uri`
- local filesystem paths
- raw remote provider URLs
- Addon tokens, provider query strings, or staging paths
- a misleading `selected` boolean when the endpoint already returns selected
  presentation images only

## Architecture Direction

### Persistence

Prefer a new Selected Artwork table/read model instead of overloading
`image_assets.selected`.

Rationale:

- `ImageAsset` currently mixes provenance, cache, selection, and public shape.
- Public selection should point at Taru-owned Managed Artwork bytes, not at a
  provider URL or cache URI.
- A stable public image ID can remain constant across reselection while the
  artifact pointer and ETag change.
- The schema can be constrained to stored artifacts and item ownership.

Expected first schema shape:

- `selected_artworks.id`
- `selected_artworks.library_id`
- `selected_artworks.item_id`
- `selected_artworks.kind`
- `selected_artworks.kind_key`
- `selected_artworks.artifact_id`
- timestamps
- unique `(item_id, kind, kind_key)`
- index on `artifact_id`

`library_id`, `item_id`, and `kind` should match the linked artifact. The
repository method should enforce this inside one transaction.

### Application Service

Add a Taru-owned publication method, likely under the artwork app service:

1. Load the stored Managed Artwork Artifact.
2. Verify it belongs to an existing item and library.
3. Upsert Selected Artwork for `(item_id, kind, kind_key)`.
4. Return a redacted publication summary and public image reference.

The first route should be an Admin API command. Automatic "select after ingest"
policy is intentionally deferred; accepting and storing an artifact should not
silently publish it.

### Public Client API

Public catalog item detail and item image listing should return the new public
image reference shape. If no Selected Artwork exists, the public image list is
empty.

`ImageAssetDto` and `ImageRefDto.uri` are not safe as-is. This lane should
either replace them in Public Client protocol types or confine them to internal
Admin/provenance surfaces. Because Taru is still early and this is a correctness
lane, prefer a clean public contract over compatibility with the old leak-prone
shape.

### Image Serving

Add a Public Client route for image bytes, for example:

```text
GET /api/v1/images/{image_id}
```

The handler should:

- require the normal Public Client auth boundary;
- load the Selected Artwork row by public image ID;
- resolve the linked Managed Artwork Artifact;
- read bytes from the internal artifact storage port using `storage_uri` only
  inside the server;
- return `Content-Type`, `Content-Length` when known, and a safe ETag;
- map missing rows to `404` without revealing whether a storage object exists;
- avoid logging source URLs, local paths, or storage URIs.

Range requests, thumbnail variants, resize parameters, and cache eviction are
deferred unless required by a focused correctness test.

## Assumptions

| Assumption | Confidence | Evidence | Mitigation |
| --- | --- | --- | --- |
| Public Client routes are already protected by the API auth boundary. | High | `access-boundary-auth` and current route structure | Add focused route tests if serving bytes introduces a new router branch. |
| Stored artifacts have enough metadata for public references before thumbnails. | High | MAFA records width, height, media type, byte length, content hash | Keep thumbnails split; serve original validated bytes first. |
| `ImageAssetDto` and `ImageRefDto.uri` are public leak risks. | High | `taru-client-protocol` and `taru-api::openapi` expose these fields | Replace the public DTO shape and add serialization tests that reject these fields. |
| Selection should be explicit, not automatic after ingest. | Medium | Previous lanes intentionally avoided publication | Add an Admin publish command first; later lanes can add policy-driven auto-selection. |

## Non-Goals And Splits

- Thumbnail generation and responsive image variants belong to a later lane.
- Durable retry/requeue and cancellation belong to managed ingest job runtime
  follow-ons.
- Orphan artifact cleanup belongs to storage lifecycle work.
- Artwork Export belongs to Library File Write policy, not public serving.
- Public gallery/candidate browsing belongs to an Admin or client UX lane after
  Selected Artwork is stable.

## Closeout Condition

This lane can close when:

- a stored Managed Artwork Artifact can be explicitly published as Selected
  Artwork;
- Public Client item image responses return only first-party redacted image
  references;
- the public image route serves bytes for selected artwork without exposing
  internal locators;
- old public `source_uri`, `cache_uri`, raw `uri`, and `selected` image fields
  are removed or confined away from Public Client DTOs;
- thumbnails, retry/requeue, cancellation, and orphan cleanup are documented as
  follow-ons with no hidden runtime dependency.
