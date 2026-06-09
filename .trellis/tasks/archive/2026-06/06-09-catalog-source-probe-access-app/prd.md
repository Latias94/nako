# refactor: move catalog source probe access into app service

## Goal

Move Public Catalog source probe browse-access enforcement from the HTTP route
helper into `CatalogAppService`, so the remaining source-shaped read surface in
`http::catalog` follows the same app-service access boundary as catalog item
detail, credits, images, list, relation, and search reads.

## What I already know

* `GET /sources/{source_id}/probe` currently calls
  `require_source_access(... Browse)` in `http::catalog`.
* `CatalogAppService::get_source_probe` currently takes only `MediaSourceId` and
  loads probe facts without checking the caller's Library Access.
* `http::access::require_source_access` is still used by playback, renderer,
  user playback, and selected-artwork byte routes; this task should not delete
  or globally replace it.
* The server HTTP spec now records Public Catalog read access as an app-service
  boundary, while selected artwork byte routes remain route-local for now.

## Assumptions

* Public source probe DTO shape stays unchanged.
* Missing media source or probe semantics stay unchanged.
* Ordinary principals may read a source probe only when they can browse the
  source library.
* Administrator principals keep existing access semantics.
* No schema, migration, repository trait, or client contract changes are needed.

## Requirements

* Make `CatalogAppService::get_source_probe` principal-aware.
* Enforce browse access in `CatalogAppService` before returning probe facts.
* Keep `http::catalog::get_source_probe` thin: parse path, pass principal to app
  service, return JSON.
* Remove `require_source_access` from `http::catalog` if no catalog route still
  needs it.
* Preserve `require_source_access` for non-catalog source read/playback routes.
* Add focused catalog HTTP tests proving hidden source probes are forbidden and
  visible source probes still return the probe DTO.

## Acceptance Criteria

* [x] `http::catalog::get_source_probe` no longer calls `require_source_access`.
* [x] `CatalogAppService::get_source_probe` enforces source Library Access
  level `browse`.
* [x] Public source probe response shape remains unchanged for visible sources.
* [x] Focused catalog HTTP tests cover visible and hidden source probe behavior.
* [x] Focused server checks and `git diff --check` pass.

## Definition of Done

* Tests added/updated where behavior is observable.
* HTTP catalog route code has no source probe access compensation.
* Trellis task is archived and session journal is updated after commit.

## Out of Scope

* Refactoring playback, renderer, user playback, selected artwork byte access,
  or metadata source/item access checks.
* Adding new repository traits or database access projection methods.
* Changing probe persistence, probe DTO fields, or image cache behavior.

## Technical Notes

* Relevant specs:
  * `.trellis/spec/nako-server/backend/http-api-patterns.md`
  * `.trellis/spec/nako-server/backend/database-guidelines.md`
  * `.trellis/spec/nako-server/backend/quality-guidelines.md`
  * `.trellis/spec/guides/cross-layer-thinking-guide.md`
  * `.trellis/spec/guides/code-reuse-thinking-guide.md`
* Relevant code:
  * `crates/nako-server/src/http/catalog.rs`
  * `crates/nako-server/src/app/catalog.rs`
  * `crates/nako-server/src/http/tests/catalog.rs`
* Architecture map: `docs/architecture/STATE_ACCESS.md`.
