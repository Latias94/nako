# Managed Artwork Public Serving Selection Milestones

Status: Completed
Last updated: 2026-05-19

## M0 - Scope And Evidence Freeze

Outcome: public managed artwork serving and Selected Artwork publication are
split from MAFA with explicit redaction boundaries.

Exit criteria:

- Problem, target state, non-goals, and closeout condition are explicit.
- MAFA closeout points to this lane as the next recommended action.
- Workstream index links the new lane.
- Thumbnail, durable retry/requeue, cancellation, and orphan cleanup are split.

Primary evidence:

- `docs/workstreams/managed-artwork-public-serving-selection/DESIGN.md`
- `docs/workstreams/managed-artwork-fetch-artifact-storage/HANDOFF.md`

## M1 - Public Contract And Selection Model Freeze

Outcome: the public DTO, route identity, and Selected Artwork persistence model
are chosen before implementation.

Exit criteria:

- `ImageAssetDto`, `ImageRefDto.uri`, OpenAPI, and catalog responses are
  audited for public leak risks.
- The public image reference fields are explicit and redacted.
- The Selected Artwork schema/repository authority is chosen.
- Old `ImageAsset` behavior is either kept internal, migrated, or scheduled for
  deletion with no public leakage.

Result:

- Public image ID authority is `selected_artworks.id`, represented in core as
  `SelectedArtworkId`.
- Public serving uses `GET /images/{image_id}` and `HEAD /images/{image_id}`;
  the route is a Catalog binary/streaming route, not a JSON method.
- Admin publication uses
  `POST /admin/v1/artwork/artifacts/{artifact_id}/publish`.
- Public Client responses use `PublicImageRefDto` with `id`, `owner`, `kind`,
  first-party relative `url`, dimensions, language, media type, and safe ETag.
- `ImageAssetDto`, `ImageRefDto.uri`, and `CanonicalMetadataDto.images` are
  removed from the Public Client protocol path during MAPS-040. Legacy
  `ImageAsset` remains internal/provenance only until a later cleanup decides
  whether to migrate or delete it.
- First migration is `0027_selected_artwork_publication.sql`, introducing
  `selected_artworks` with stable public IDs and unique `(item_id, kind,
  kind_key)` selection slots.

Primary gates:

- `rg -n "ImageAssetDto|ImageRefDto|source_uri|cache_uri|storage_uri|selected|managed_artwork_artifacts|list_item_images|/items/\\{item_id\\}/images|/images" crates docs`
- `git diff --check`

## M2 - Selected Artwork Publication

Outcome: a stored Managed Artwork Artifact can be explicitly published as the
current item/kind presentation image.

Exit criteria:

- Schema and repository methods persist Selected Artwork idempotently.
- Admin publication verifies the artifact is stored and belongs to the target
  item/kind.
- Publication responses expose only safe IDs, kind, dimensions, media type, and
  public image reference fields.
- No source URL, cache URI, storage URI, or local path appears in Admin or
  Public Client responses.

Primary gates:

- focused db publication tests
- focused admin publication HTTP tests
- `cargo check -p nako-core -p nako-db -p nako-api -p nako-server --tests`
- `cargo fmt --all -- --check`
- `git diff --check`

Result:

- Added `SelectedArtworkId` and Selected Artwork publication records to
  `nako-core`.
- Added `0027_selected_artwork_publication.sql` with stable
  `selected_artworks.id`, unique `(item_id, kind, kind_key)` selection slots,
  and `ON DELETE RESTRICT` artifact references.
- Added repository methods for idempotent publish, selected-artwork lookup, and
  item selected-artwork listing.
- Added redacted Admin publication response with `PublicImageRefDto`.
- Added `POST /admin/v1/artwork/artifacts/{artifact_id}/publish`, proving a
  stored Managed Artwork Artifact can become Selected Artwork without exposing
  `storage_uri`, local paths, source URLs, `cache_uri`, or addon token material.

## M3 - Public Image References And Byte Serving

Outcome: clients can discover and fetch the selected image through Nako-owned
public routes.

Exit criteria:

- Public item detail and item image listing return only redacted first-party
  image references.
- A public image route streams bytes for selected artwork from internal storage.
- Missing or unpublished artwork does not reveal internal storage existence.
- OpenAPI and client protocol types match the new redacted contract.

Primary gates:

- focused catalog/image HTTP tests
- OpenAPI schema and route inventory tests
- `cargo nextest run -p nako-server image --no-fail-fast`
- `cargo nextest run -p nako-api image --no-fail-fast`
- `git diff --check`

Result:

- Public Client `ItemDetailResponse.images` and `ImagesResponse.images` now
  use `PublicImageRefDto`.
- `CanonicalMetadataDto.images`, public `ImageAssetDto`, and public
  `ImageRefDto` were removed from the Public Client protocol and OpenAPI
  contract.
- `GET /images/{image_id}` and `HEAD /images/{image_id}` were added as
  Catalog streaming routes in the protocol inventory, OpenAPI, Rust SDK request
  builders, TypeScript SDK runtime, and HTTP router.
- `CatalogAppService` reads `selected_artworks` and returns first-party image
  references; old `ImageAsset` rows remain internal/provenance only.
- `ManagedArtworkAppService` reads selected artifact bytes through internal
  Managed Artwork Artifact storage without exposing `storage_uri`, local paths,
  raw provider URLs, `source_uri`, or `cache_uri`.

## M4 - Closeout Or Split

Outcome: the public serving/selection boundary is complete or remaining lifecycle
work is split into narrower lanes.

Exit criteria:

- Fresh command evidence is recorded.
- HTTP/API docs reflect shipped behavior.
- Public Client leak inventory proves raw image locator fields are absent from
  public contracts.
- Thumbnails, durable retry/requeue, cancellation, orphan cleanup, and public
  gallery behavior are completed, deferred, or split.

Result:

- MAPS is closed as completed: Selected Artwork publication, public image
  references, byte serving, and redaction boundaries are implemented and
  documented.
- Fresh closeout gates passed on 2026-05-19.
- Remaining lifecycle work is intentionally split:
  - thumbnail/resize variants;
  - durable retry/requeue and cancellation controls;
  - orphan artifact cleanup and retention diagnostics;
  - public/Admin gallery and candidate-management behavior.
