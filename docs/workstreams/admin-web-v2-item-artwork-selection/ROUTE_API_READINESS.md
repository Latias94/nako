# Admin Web V2 Item Artwork Selection - Route/API Readiness

Status: Accepted With Contract Work
Last updated: 2026-05-25
Task: AWA-020

## Readiness Claim

The backend item artwork gallery/select/unpublish routes are ready for an
Admin Web one-item artwork workflow, but the generated Admin Web contract is
not ready. AWA-030 must add generated route constants and TypeScript DTOs before
UI implementation.

No backend/API DTO hardening blocker was found for this first slice. Existing
HTTP docs and server/API tests prove that the gallery, select, and unpublish
responses omit source/storage/cache/path/hash/token material. Admin Web must
still project those responses into route-local summaries rather than rendering
whole JSON payloads.

## Route Inventory

In scope for this lane:

| Route | Method | Purpose | Request | Response |
| --- | --- | --- | --- | --- |
| `/admin/v1/items/{item_id}/artwork?limit=50&offset=0` | GET | Item-scoped Managed Artwork gallery. | `AdminPageQuery` params. | `AdminManagedArtworkGalleryResponse`. |
| `/admin/v1/items/{item_id}/artwork/{kind}/select` | POST | Select or replace one item/kind Selected Artwork slot from a stored artifact. | `{ "artifact_id": "<ManagedArtworkArtifactId>" }`. | `PublishSelectedArtworkResponse`. |
| `/admin/v1/items/{item_id}/artwork/{kind}/selection` | DELETE | Unpublish one item/kind Selected Artwork slot. | none. | `UnpublishSelectedArtworkResponse`. |

Supported `kind` path values are:

- `poster`
- `backdrop`
- `logo`
- `thumbnail`
- `banner`

Out of scope for this lane:

- `POST /admin/v1/artwork/candidates/{candidate_id}/accept`
- `POST /admin/v1/artwork/ingests/process-next`
- `POST /admin/v1/artwork/ingests/{ingest_id}/requeue`
- `POST /admin/v1/artwork/artifacts/{artifact_id}/publish`
- artifact lifecycle, storage drift, remediation, cleanup, thumbnail eviction,
  provider search, upload, or re-ingest workflows

## DTO Safety

Accepted safe fields for Admin Web rendering:

- IDs: item, library, candidate, side effect, addon, ingest, artifact, selected
  artwork, public image ID.
- Image kind, source kind, candidate status, ingest status, selected flags, and
  selected counts.
- Dimensions, byte length, media type, language, created/updated timestamps.
- `has_content_hash` as a boolean only.
- First-party relative image routes such as `/images/{image_id}`.
- Result `changed` state for select and unpublish.

Forbidden rendered fields and values:

- `source_uri`
- `storage_uri`
- `managed-artwork://...`
- cache URIs
- local paths and artifact roots
- raw provider URLs or query strings
- raw Addon/provider payloads
- file contents
- `content_hash` values
- tokens, credentials, and secret-like values

Evidence:

- `docs/api/HTTP_API.md` documents the redaction contract for gallery, select,
  and unpublish.
- `crates/nako-api/src/admin/managed_artwork.rs` serializes summaries with
  safe IDs/counts/flags and `has_content_hash`, not the actual hash or storage
  URI.
- `crates/nako-server/src/http/tests/addons.rs` covers gallery, selection, and
  unpublish redaction against remote URLs, token material, `source_uri`,
  `cache_uri`, `storage_uri`, `managed-artwork://`, local paths, and
  `content_hash`.

## Generated Contract Gap

`apps/admin-web/src/adminApi/generated/contract.ts` currently has no item
artwork route constants and no `AdminManagedArtwork*` DTOs. The Rust contract
source in `crates/nako-api/src/admin_contract.rs` also lacks those route
suffixes and TypeScript interfaces.

AWA-030 must add, then regenerate:

- route constants:
  - `itemArtworkGallery`
  - `itemArtworkSelect`
  - `itemArtworkSelection`
- query/request types:
  - `AdminItemArtworkGalleryQuery extends AdminPageQuery`
  - `AdminSelectItemArtworkRequest`
  - `AdminArtworkKind`
- response/summary types:
  - `AdminManagedArtworkGalleryResponse`
  - `AdminManagedArtworkGallerySummary`
  - `AdminManagedArtworkGalleryCandidate`
  - `AdminManagedArtworkGalleryArtifact`
  - `AdminManagedArtworkGallerySelected`
  - `ManagedArtworkIngestSummary`
  - `SelectedArtworkSummary`
  - `PublishSelectedArtworkResponse`
  - `UnpublishSelectedArtworkResponse`
  - `UnpublishedSelectedArtworkSummary`

`PublicImageRef` already exists in Admin Web local public bridge types; AWA-030
can either reuse a compatible exported type or define a contract-local
`PublicImageRefDto` shape matching current generated Admin DTO needs.

## UI Readiness Rules

- Gallery reads may use deterministic mock fallback when the live Admin API is
  unavailable.
- Select and unpublish mutations must never report fake successful results.
- Mutation failures, including `400` unsupported kind and `409` item/kind or
  artifact eligibility conflicts, must be visible to the operator.
- The UI should require explicit prepare/confirm flow before select/replace or
  unpublish.
- The gallery route should be reachable from `/items/:itemId` and keep
  pagination route-owned.
- Image URLs must be rendered only when they are first-party relative
  `/images/...` paths; unsafe absolute or storage/cache URLs must be ignored.

## Decision

Proceed to AWA-030 generated contract coverage. Do not start the artwork UI
until contract constants and DTOs exist and client tests prove the generated
paths, request body, and delete route behavior.
