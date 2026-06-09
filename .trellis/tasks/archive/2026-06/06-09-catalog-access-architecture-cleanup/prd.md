# refactor: clean catalog access architecture

## Goal

Remove the remaining HTTP-layer catalog access probe for person detail routes
and keep Public Catalog access semantics inside the catalog app/repository
boundary before starting new feature work.

## What I already know

* The previous task moved `/people`, `/tags`, and `/genres` root aggregates to
  repository-backed access-before-pagination queries.
* `/people/{person_id}` still performs a bespoke HTTP-layer check by loading a
  full first page of unfiltered person items and then probing each item access.
* Catalog relation item routes already use `list_accessible_*_items`, so the
  repository has the right access-before-pagination primitive for this check.
* HTTP route specs require handlers to stay thin and delegate application
  semantics to app services.
* Database specs require bounded repository-backed browse/access queries and
  warn against app/HTTP access-after-pagination behavior.

## Assumptions

* Keep the existing public route shape and DTOs unchanged.
* Keep administrator behavior for orphan person records: admins can still read
  person detail records even when no accessible media item proves visibility.
* For non-admin principals, a person detail record is visible only when at
  least one related media item is accessible through browse-level Library
  Access.
* Do not add a new repository trait method in this cleanup slice; use the
  existing `list_accessible_person_items` contract with a one-row page probe.

## Requirements

* Delete the private HTTP helper that performs unfiltered person item access
  probing.
* Move `/people/{person_id}` authorization into `CatalogAppService`.
* Use a bounded repository-backed access probe for non-admin person detail
  reads.
* Preserve `404` for missing person records.
* Preserve `403` for an existing person with only inaccessible related items.
* Preserve administrator access to orphan person records.

## Acceptance Criteria

* [x] `crates/nako-server/src/http/catalog.rs` no longer has
  `person_has_accessible_item` or `any_public_items_accessible`.
* [x] `CatalogAppService::get_person` accepts the principal and owns the access
  decision.
* [x] A focused HTTP regression proves hidden person detail reads are forbidden
  while visible person detail reads still work.
* [x] Focused catalog route tests pass.
* [x] `cargo fmt --all`, focused checks/tests, and `git diff --check` pass.

## Verification

* `cargo fmt --all`
* `cargo check -p nako-server --tests`
* `cargo nextest run -p nako-server http::tests::catalog --no-fail-fast`
* `git diff --check`

## Definition of Done

* Tests added/updated where behavior is user-visible.
* Lint/typecheck/test gates pass for the touched crates.
* Trellis task is archived and session journal is updated after commit.

## Out of Scope

* Reworking `/search` to fully repository-backed access-before-pagination.
* Adding tag/genre detail routes.
* Adding new repository trait methods for single-record aggregate visibility.
* Changing public client protocol, OpenAPI, or DTO shapes.
* Changing database schema or migrations.

## Technical Notes

* Relevant specs:
  * `.trellis/spec/nako-server/backend/http-api-patterns.md`
  * `.trellis/spec/nako-server/backend/database-guidelines.md`
  * `.trellis/spec/nako-server/backend/quality-guidelines.md`
  * `.trellis/spec/nako-db/backend/database-guidelines.md`
  * `.trellis/spec/nako-db/backend/quality-guidelines.md`
  * `.trellis/spec/nako-core/backend/database-guidelines.md`
  * `.trellis/spec/guides/cross-layer-thinking-guide.md`
  * `.trellis/spec/guides/code-reuse-thinking-guide.md`
* Architecture map: `docs/architecture/STATE_ACCESS.md`.
* Existing DB contracts already prove relation and root aggregate access
  behavior across SQLite/PostgreSQL.
