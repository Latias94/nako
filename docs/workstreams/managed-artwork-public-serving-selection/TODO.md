# Managed Artwork Public Serving Selection TODO

Status: Active
Last updated: 2026-05-19

## M0 - Scope And Evidence Freeze

- [x] MAPS-010 [owner=planner] [deps=MAFA-050] [scope=docs/workstreams/managed-artwork-public-serving-selection,docs/workstreams/managed-artwork-fetch-artifact-storage,docs/workstreams/README.md]
  Goal: Open a focused follow-on for turning stored Managed Artwork Artifacts
  into Selected Artwork and Public Client image references without leaking raw
  storage/source/cache locators.
  Validation: `Get-Content -Raw docs/workstreams/managed-artwork-public-serving-selection/WORKSTREAM.json | ConvertFrom-Json | Out-Null`; `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, MAFA closeout docs.
  Result: DONE. The lane is split from MAFA and keeps thumbnails, durable
  retry/requeue, cancellation, and orphan artifact cleanup out of the first
  public-serving path.
  Handoff: Continue with MAPS-020 before changing protocol DTOs or adding
  schema migrations.

## M1 - Public Contract And Selection Model Freeze

- [x] MAPS-020 [owner=codex] [deps=MAPS-010] [scope=crates/taru-core,crates/taru-api,crates/taru-client-protocol,crates/taru-server,docs/api,docs/workstreams/managed-artwork-public-serving-selection]
  Goal: Freeze the public image reference DTO and Selected Artwork persistence
  model, including how `ImageAssetDto`, `ImageRefDto.uri`, OpenAPI, and catalog
  responses stop exposing raw image locators.
  Validation: `rg -n "ImageAssetDto|ImageRefDto|source_uri|cache_uri|storage_uri|selected|managed_artwork_artifacts|list_item_images|/items/\\{item_id\\}/images|/images" crates docs`; `git diff --check`.
  Review: choose the public image ID authority, route shape, DTO fields, and
  migration contract before implementation. Confirm whether old `ImageAsset`
  remains internal, is migrated, or is deleted in this lane.
  Evidence: design update and audit notes in `EVIDENCE_AND_GATES.md`.
  Result: DONE. Public image ID authority is `selected_artworks.id`; public
  byte route is `GET/HEAD /images/{image_id}`; Admin publication route is
  `POST /admin/v1/artwork/artifacts/{artifact_id}/publish`; public DTO is
  `PublicImageRefDto`; first migration is `0027_selected_artwork_publication`;
  old `ImageAsset` remains internal/provenance only and must be removed from
  Public Client DTO/OpenAPI paths during MAPS-040.
  Handoff: Continue with MAPS-030 once the public DTO and Selected Artwork
  schema shape are no longer ambiguous.

## M2 - Selected Artwork Publication

- [x] MAPS-030 [owner=codex] [deps=MAPS-020] [scope=crates/taru-core,crates/taru-db,crates/taru-api,crates/taru-server,docs/api]
  Goal: Publish one stored Managed Artwork Artifact as the Selected Artwork for
  its item/kind through an explicit Admin API command.
  Validation: focused db publication tests; focused admin HTTP tests; `cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: the publication command must be idempotent, constrained to stored
  artifacts, and redacted. It must not expose `storage_uri`, source URL,
  `cache_uri`, local path, or Addon/provider token material.
  Evidence: migration/repository/service/API tests and notes in
  `EVIDENCE_AND_GATES.md`.
  Result: DONE. Added `SelectedArtworkId`, `selected_artworks` migration,
  selected-artwork repository publish/read methods, `PublicImageRefDto`,
  redacted `PublishSelectedArtworkResponse`, and
  `POST /admin/v1/artwork/artifacts/{artifact_id}/publish`. Publication is
  idempotent per item/kind slot and preserves a stable public selected-artwork
  ID across replay.
  Handoff: Continue with MAPS-040 after a selected record can be created and
  read without serving bytes yet.

## M3 - Public Image References And Byte Serving

- [ ] MAPS-040 [owner=codex] [deps=MAPS-030] [scope=crates/taru-core,crates/taru-db,crates/taru-api,crates/taru-client-protocol,crates/taru-server,docs/api]
  Goal: Return first-party Public Client image references for selected artwork
  and serve selected image bytes through a Public Client route.
  Validation: focused catalog/image HTTP tests; OpenAPI route/schema tests; `cargo nextest run -p taru-server image --no-fail-fast`; `cargo nextest run -p taru-api image --no-fail-fast`; `git diff --check`.
  Review: item detail and item image responses must not include `source_uri`,
  `cache_uri`, `storage_uri`, raw `uri`, local paths, or stale `selected`
  booleans. The serving route must resolve storage only inside the server
  boundary. `ImageAssetDto` and `ImageRefDto` must be deleted or made
  non-public; `CanonicalMetadataDto` must not serialize provider image URIs.
  Evidence: route tests, protocol/OpenAPI tests, and redaction inventory in
  `EVIDENCE_AND_GATES.md`.
  Handoff: Continue with MAPS-050 to close or split remaining image lifecycle
  work.

## M4 - Closeout Or Split

- [ ] MAPS-050 [owner=planner] [deps=MAPS-040] [scope=docs/workstreams/managed-artwork-public-serving-selection,docs/api]
  Goal: Close the lane or split thumbnails, durable retry/requeue,
  cancellation, orphan cleanup, and public gallery behavior into separate
  follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings; Public Client leak
  inventory proves old raw image locator fields are absent or non-public.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Recommend the next lane only after Selected Artwork publication and
  public serving redaction are stable.
