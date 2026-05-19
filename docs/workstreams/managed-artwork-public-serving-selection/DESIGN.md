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

### MAPS-020 Frozen Contract

The first implementation target is now fixed as follows:

- Public image ID authority: `selected_artworks.id`.
- Public image byte route: `GET /images/{image_id}` and
  `HEAD /images/{image_id}`.
- Admin publication route:
  `POST /admin/v1/artwork/artifacts/{artifact_id}/publish`.
- Public DTO: `PublicImageRefDto`.
- Admin publication DTO: `PublishSelectedArtworkResponse`.
- Core ID: `SelectedArtworkId`.
- First persistence migration: `0027_selected_artwork_publication.sql`.

This contract deliberately makes Selected Artwork the public image identity.
Clients never receive `ManagedArtworkArtifactId` as a fetch URL, never receive
`managed-artwork://...`, and never need to know whether bytes live in local
artifact storage, a future object store, or a thumbnail pipeline.

### Persistence

Use a new Selected Artwork table/read model instead of overloading
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

`selected_artworks.id` is stable for the `(item_id, kind, kind_key)` selection
slot. Reselecting a different artifact for the same item/kind updates
`artifact_id` and `updated_at` while preserving the public image URL. A changed
ETag tells clients that the bytes behind the stable route changed.

`artifact_id` should use `ON DELETE RESTRICT`. Orphan cleanup must never delete
currently selected artwork by accident; cleanup can only remove unselected
artifacts in a later lifecycle lane.

`library_id`, `item_id`, and `kind` must match the linked artifact. SQLite
cannot express that cross-table invariant as a simple check constraint, so the
repository publish method must enforce it inside one transaction before the
upsert.

First repository methods:

- `publish_selected_artwork(artifact_id) -> SelectedArtworkPublicationRecord`
- `get_selected_artwork(id) -> Option<SelectedArtworkRecord>`
- `list_selected_artwork_for_item(item_id) -> Vec<SelectedArtworkRecord>`

The publication method must require a stored artifact row. Failed ingests,
candidate rows, raw provider images, and legacy `ImageAsset` rows are not
publishable.

### Application Service

Add a Taru-owned publication method, likely under the artwork app service:

1. Load the stored Managed Artwork Artifact.
2. Verify it belongs to an existing item and library.
3. Upsert Selected Artwork for `(item_id, kind, kind_key)`.
4. Return a redacted publication summary and public image reference.

The Admin route is:

```text
POST /admin/v1/artwork/artifacts/{artifact_id}/publish
```

It has no body in the first slice. The artifact already carries the item,
library, and image kind. A future policy route can add body fields for
alternate owners, variants, or batch selection, but the first boundary should
not accept raw paths, source URLs, storage URIs, or arbitrary public URLs.

Automatic "select after ingest" policy is intentionally deferred; accepting and
storing an artifact should not silently publish it.

The response shape is:

- `selected_artwork.id`
- `selected_artwork.library_id`
- `selected_artwork.item_id`
- `selected_artwork.kind`
- `selected_artwork.artifact_id`
- `selected_artwork.created_at`
- `selected_artwork.updated_at`
- `image: PublicImageRefDto`
- `changed: bool`

`artifact_id` is safe as an Admin identifier, but it is not a fetch URL. The
response must omit `storage_uri`, local paths, raw source URLs, `cache_uri`,
candidate `source_uri`, and provider/addon token material.

### Public Client API

Public catalog item detail and item image listing should return the new public
image reference shape. If no Selected Artwork exists, the public image list is
empty.

`ImageAssetDto` and `ImageRefDto.uri` are not safe as-is and must be removed
from the Public Client protocol surface in this lane. `ImageAsset` may remain
as an internal catalog/provenance record while provider metadata ingestion still
uses it, but no Public Client DTO or OpenAPI schema should serialize it.

Public protocol changes:

- Replace `ItemDetailResponse.images: Vec<ImageAssetDto>` with
  `Vec<PublicImageRefDto>`.
- Replace `ImagesResponse.images: Vec<ImageAssetDto>` with
  `Vec<PublicImageRefDto>`.
- Remove `CanonicalMetadataDto.images` from Public Client responses for this
  lane. Canonical Metadata should not carry provider image URLs into client
  protocol DTOs. A future browse summary can add selected public artwork
  explicitly if list/search cards need images.
- Delete or make non-public `ImageAssetDto` and `ImageRefDto` from
  `taru-client-protocol`.
- Update OpenAPI to define `PublicImageRefDto`, remove the old image URI/cache
  schemas, and add `/images/{image_id}` as a binary response route.

`PublicImageRefDto` fields:

```text
id: string
owner: ClientImageOwner
kind: ClientImageKind
url: string
width: number | null
height: number | null
language: string | null
media_type: string | null
etag: string | null
```

The `url` must be a first-party relative route such as
`/images/{image_id}`. It must never be a provider URL, local path,
`managed-artwork://...` URI, cache URI, Source Locator, data URI, or temporary
signed URL.

### Image Serving

Add a Public Client route for image bytes, for example:

```text
GET  /images/{image_id}
HEAD /images/{image_id}
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

The route is a Catalog route in the public route inventory but should be marked
as streaming/binary, not as a JSON method. The generated Rust SDK should expose
it with a streaming builder similar to playback byte routes.

`LocalManagedArtworkArtifactStore` currently supports write and best-effort
delete only. MAPS-040 must add a read/stream helper that resolves bytes from the
artifact ID and stored media type under the configured artifact root. It should
validate the expected `managed-artwork://artifact/{artifact_id}` shape but must
not expose or return the filesystem path. The first implementation may read the
whole validated image into memory because MAFA already enforces the artwork byte
limit; range support remains a follow-on.

Range requests, thumbnail variants, resize parameters, and cache eviction are
deferred unless required by a focused correctness test.

### Legacy ImageAsset Policy

`ImageAsset` remains an internal catalog/provenance model during this lane. It
is still used by `taru-catalog` and existing catalog repository tests for
provider/local image facts. It is not the public selected-artwork authority.

MAPS-040 should remove these public adapter paths:

- `taru_api::image_asset_to_dto`
- `ImageAssetDto` from `taru-client-protocol`
- `ImageRefDto.uri` from `CanonicalMetadataDto`
- OpenAPI `ImageAssetDto` and `ImageRefDto` schemas
- Public catalog responses backed by `store.list_item_images`

After the public serving path is stable, a later cleanup can decide whether
legacy `image_assets.selected` should be deleted, preserved only for internal
metadata provenance, or migrated into Artwork Candidate/Managed Artwork flows.

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
