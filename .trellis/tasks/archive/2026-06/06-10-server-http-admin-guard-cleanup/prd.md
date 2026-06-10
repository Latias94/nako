# refactor: remove final http access admin guard module

## Goal

Finish the `nako-server` HTTP access-boundary cleanup by removing the final
generic `http/access.rs` helper module after Library Access checks were moved
into app services, while preserving existing Metadata route administrator
behavior.

## What I already know

* `http/access.rs` now contains only `require_administrator`.
* The only caller is `crates/nako-server/src/http/metadata.rs`.
* The remaining guarded routes are:
  `/metadata/providers`,
  `/metadata/maintenance/jobs`,
  `/metadata/maintenance/plan`, and `/metadata/raw/cleanup`.
* `http/admin.rs` already has a separate Admin API route-layer guard for
  `/admin/v1/*` and should not be changed in this cleanup.
* Moving the guard into middleware could change extractor/error priority for
  malformed JSON/query requests, so this task should keep the same inline
  guard timing.

## Requirements

* Delete the obsolete `http/access.rs` module.
* Move the administrator helper into `http/metadata.rs` as a local route helper
  or equivalent inline function.
* Preserve the existing forbidden error message:
  `administrator role is required`.
* Preserve route paths, DTOs, status codes, and handler extractor order.
* Do not change `/admin/v1/*` Admin route-layer behavior.

## Acceptance Criteria

* [x] `crates/nako-server/src/http.rs` no longer declares `mod access`.
* [x] `crates/nako-server/src/http/access.rs` is deleted.
* [x] `http/metadata.rs` compiles without importing `super::access`.
* [x] Existing metadata HTTP tests pass.
* [x] `cargo check -p nako-server --tests` passes.

## Definition of Done

* [x] Rust code is formatted with `cargo fmt --all`.
* [x] Focused metadata HTTP tests pass.
* [x] `cargo check -p nako-server --tests` passes.
* [x] `git diff --check` passes.
* [x] Relevant Trellis spec or task notes are updated.
* [ ] Task is archived, journal is recorded, commits are pushed.

## Out of Scope

* Moving metadata maintenance routes under `/admin/v1`.
* Replacing inline guards with middleware route layers.
* Changing Admin API route guard behavior.
* Changing Metadata app-service authorization behavior.
* Public API or DTO shape changes.

## Technical Approach

* Add a private `require_administrator` helper to `http/metadata.rs`.
* Remove `use super::access::require_administrator`.
* Delete `http/access.rs` and remove `mod access` from `http.rs`.
* Run metadata-focused tests and server check.

## Technical Notes

* Relevant files:
  `crates/nako-server/src/http.rs`,
  `crates/nako-server/src/http/access.rs`,
  `crates/nako-server/src/http/metadata.rs`,
  `crates/nako-server/src/http/tests/metadata.rs`.
* Relevant specs:
  `.trellis/spec/nako-server/backend/http-api-patterns.md`,
  `.trellis/spec/nako-server/backend/error-handling.md`,
  `.trellis/spec/nako-server/backend/quality-guidelines.md`,
  `.trellis/spec/nako-server/backend/directory-structure.md`.
