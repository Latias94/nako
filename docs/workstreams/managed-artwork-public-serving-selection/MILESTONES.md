# Managed Artwork Public Serving Selection Milestones

Status: Active
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
- `cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests`
- `cargo fmt --all -- --check`
- `git diff --check`

## M3 - Public Image References And Byte Serving

Outcome: clients can discover and fetch the selected image through Taru-owned
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
- `cargo nextest run -p taru-server image --no-fail-fast`
- `cargo nextest run -p taru-api image --no-fail-fast`
- `git diff --check`

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
