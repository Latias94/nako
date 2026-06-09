# refactor: move catalog item detail access into app service

## Goal

Move Public Catalog item detail access enforcement and visible-source shaping
from HTTP route helpers into `CatalogAppService`, so item detail, credits, and
images read surfaces share the same app-service access boundary as catalog
list/search/relation projections.

## What I already know

* `/items/{item_id}`, `/items/{item_id}/credits`, and
  `/items/{item_id}/images` currently call `require_item_access` in
  `http::catalog`.
* `/items/{item_id}` also calls `filter_item_detail_sources` in
  `http::catalog` to remove inaccessible source DTOs after
  `CatalogAppService::get_item` has already built the detail response.
* Recent catalog access work moved relation list, root aggregate, person
  detail, and search access behavior into repository/app-service boundaries.
* `http::access::require_item_access` is still used by user playback,
  playlist, metadata, and other non-catalog write/read surfaces; this task
  should not delete it globally.

## Assumptions

* Public `/items/*` route shape and DTO fields stay unchanged.
* Administrator principals keep existing source-less item semantics.
* Ordinary principals see an item only when it has at least one accessible
  media source through Library Access.
* Source filtering for item detail should happen before DTO construction inside
  the app service, not after DTO construction in the HTTP layer.
* No schema or migration changes are needed.

## Requirements

* Add principal-aware catalog app-service methods for item detail, item credits,
  and item images.
* Ensure `CatalogAppService::get_item` for public use only returns item detail
  when the principal can browse the item.
* Ensure item detail source DTOs include only sources visible to the principal.
* Keep credits/images responses inaccessible for hidden items.
* Keep HTTP catalog handlers thin: parse path/query, pass principal to app
  service, return JSON.
* Remove route-local item-detail source filtering from `http::catalog` once the
  app service owns it.
* Preserve `require_item_access` for non-catalog routes that still own their
  own write semantics in this slice.

## Acceptance Criteria

* [x] `http::catalog::{get_item,list_item_credits,list_item_images}` no longer
  call `require_item_access`.
* [x] `http::catalog` no longer has `filter_item_detail_sources` or equivalent
  post-DTO source filtering.
* [x] `CatalogAppService` enforces item browse access before returning detail,
  credits, or images.
* [x] Item detail source filtering happens inside `CatalogAppService`.
* [x] Focused catalog HTTP tests prove hidden item detail/credits/images are
  forbidden and visible item detail does not include hidden sources.
* [x] Focused server checks and `git diff --check` pass.

## Definition of Done

* Tests added/updated where behavior is observable.
* HTTP route code is thinner than before and has no catalog item detail access
  compensation.
* Trellis task is archived and session journal is updated after commit.

## Out of Scope

* Refactoring playback, playlist, or metadata route access checks.
* Adding new repository traits or database access projection methods.
* Changing public DTO shape, OpenAPI/client contracts, or image cache behavior.
* Reworking selected artwork byte access.

## Technical Notes

* Relevant specs:
  * `.trellis/spec/nako-server/backend/http-api-patterns.md`
  * `.trellis/spec/nako-server/backend/database-guidelines.md`
  * `.trellis/spec/nako-server/backend/quality-guidelines.md`
  * `.trellis/spec/nako-db/backend/quality-guidelines.md`
  * `.trellis/spec/guides/cross-layer-thinking-guide.md`
  * `.trellis/spec/guides/code-reuse-thinking-guide.md`
* Relevant code:
  * `crates/nako-server/src/http/catalog.rs`
  * `crates/nako-server/src/app/catalog.rs`
  * `crates/nako-server/src/http/tests/catalog.rs`
* Architecture map: `docs/architecture/STATE_ACCESS.md`.
