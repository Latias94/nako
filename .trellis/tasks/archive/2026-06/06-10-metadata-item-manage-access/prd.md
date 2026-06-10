# refactor: move metadata item manage access into app service

## Goal

Move item-scoped metadata `Manage` library access enforcement out of HTTP route handlers and into `MetadataAppService`, so metadata read/refresh surfaces own their authorization at the same layer that loads metadata diagnostics, raw responses, and candidate reviews.

## What I already know

* The previous subtitle and renderer transport slices moved source `Play` checks from HTTP-only helpers into app services.
* `crates/nako-server/src/http/metadata.rs` is now the only non-library route slice still calling `require_item_access(... Manage)`.
* `http::access::require_item_access` is still needed by neither metadata nor catalog after this slice if no other route uses it.
* Metadata item routes are Public Client routes, while maintenance/raw cleanup/provider diagnostics routes remain administrator-only.
* Current metadata item route behavior requires `Manage` access on at least one source library for ordinary users; source-less items are administrator-only.

## Assumptions

* Metadata refresh, item provider attempts, item raw responses, and item candidate review should use the same `Manage` item access rule.
* Administrator-only metadata maintenance/provider routes should keep route-local administrator checks because those routes are global/admin surfaces, not item-scoped library authorization.
* Internal metadata command helpers can remain available without a principal where they are not HTTP-facing.

## Requirements

* HTTP metadata item handlers must pass `AuthenticatedPrincipal` to `MetadataAppService` instead of calling `require_item_access`.
* `MetadataAppService` must reject ordinary principals without item `Manage` access before enqueueing refresh jobs or exposing item diagnostics/candidates.
* Preserve existing error behavior: insufficient item library access returns `NakoError::Forbidden` with required Library Access level `manage`; unknown/source-less items remain administrator-only before item diagnostics are exposed.
* Remove route-local item access helper code if it becomes unused.
* Keep administrator-only metadata maintenance/provider routes unchanged unless a direct cleanup is required.

## Acceptance Criteria

* [x] Browse-only principals are rejected for at least one metadata item route through the app-service access boundary.
* [x] Existing metadata refresh/diagnostics/candidate route tests remain green.
* [x] `require_item_access` is removed if it has no remaining source references.
* [x] Server focused tests and `cargo check -p nako-server --tests` pass.
* [x] Relevant Trellis spec is updated if the metadata exception in HTTP API patterns is no longer true.

## Definition of Done

* [x] Rust code is formatted with `cargo fmt --all`.
* [x] Focused `cargo nextest run -p nako-server metadata --no-fail-fast` passes.
* [x] `cargo check -p nako-server --tests` passes.
* [x] `git diff --check` passes.
* [ ] Task notes/spec/journal are updated, then changes are committed and pushed.

## Out of Scope

* Changing metadata maintenance admin authorization.
* Changing candidate review business rules, provider search behavior, or raw response cleanup semantics.
* Adding new metadata API surfaces.

## Technical Notes

* Relevant specs: `.trellis/spec/nako-server/backend/http-api-patterns.md`, `error-handling.md`, `quality-guidelines.md`, and shared cross-layer thinking guide.
* Expected code paths: `crates/nako-server/src/http/metadata.rs`, `crates/nako-server/src/app/metadata.rs`, `crates/nako-server/src/http/access.rs`, and metadata-focused tests.
