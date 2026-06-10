# refactor: move library manage command access into app services

## Goal

Continue the server access-boundary cleanup by moving Public Library
Manage-command authorization out of `http/library.rs` and into the app-service
entry points that own each command, while preserving existing public API
status codes and job behavior.

## What I already know

* The previous slice moved Public Library browse/read authorization into
  `LibraryAppService` and left Public Library Manage commands intentionally out
  of scope.
* Remaining public Library Manage route-local checks are concentrated in
  `crates/nako-server/src/http/library.rs`.
* Current routes using `require_library_manage_access` are:
  `/libraries/{library_id}/scan`,
  `/libraries/{library_id}/nfo/import`,
  `/libraries/{library_id}/nfo/export`,
  `/libraries/{library_id}/ingestion/failures` GET, and
  `/libraries/{library_id}/ingestion/failures` POST.
* Owning app services are split by command:
  `LibraryScanAppService` in `app/jobs.rs`,
  `NfoAppService` in `app/nfo.rs`, and
  `LibraryAppService` in `app/library.rs`.
* Admin library NFO command routes already rely on the admin route layer and
  are not Public Client Library Access routes.

## Assumptions

* Public Client Manage authorization should be enforced by the owning app
  service, not by HTTP route helpers.
* Raw/internal app-service methods should remain available where startup,
  tests, admin routes, or background flows need command behavior without a
  Public Client principal.
* The temporary HTTP manage guard can be deleted once all Public Library
  command routes stop using it.

## Requirements

* Move Public Library scan Manage access into the library scan app-service
  command boundary.
* Move Public Library NFO import/export Manage access into the NFO app-service
  command boundary.
* Move ingestion failure list/ignore Manage access into `LibraryAppService`.
* Keep HTTP handlers thin: parse request state/path/query/body, build trace
  context where HTTP-specific, pass `AuthenticatedPrincipal` to app services,
  and return DTOs.
* Preserve current error semantics for insufficient Manage access:
  `NakoError::Forbidden` with required Library Access level `manage`.
* Preserve existing job enqueue, trace context, NFO resource class, ingestion
  failure filtering, and response DTO behavior.
* Retain raw/internal app-service methods when existing non-HTTP callers need
  them.

## Acceptance Criteria

* [x] `http/library.rs` no longer calls `require_library_manage_access`.
* [x] Public Library Manage routes pass `AuthenticatedPrincipal` into the
      owning app service.
* [x] App-service tests cover Manage denial for scan, NFO import/export, and
      ingestion failure list/ignore where practical.
* [x] Existing HTTP library command tests still pass.
* [x] The temporary HTTP library manage guard is removed if no longer used.
* [x] Focused `cargo nextest` and `cargo check -p nako-server --tests` pass.

## Definition of Done

* [x] Rust code is formatted with `cargo fmt --all`.
* [x] Focused server tests for scan/NFO/ingestion command access pass.
* [x] `cargo check -p nako-server --tests` passes.
* [x] `git diff --check` passes.
* [x] Relevant Trellis spec or task notes are updated.
* [ ] Task is archived, journal is recorded, commits are pushed.

## Out of Scope

* Admin route authorization changes.
* Public API DTO or route shape changes.
* Background job runtime redesign.
* Schema migrations.
* Broad Library Access model redesign.

## Technical Approach

* Add Public Client Manage wrapper methods that accept
  `&AuthenticatedPrincipal` in the owning app services.
* Reuse the existing effective Library Access repository path and the standard
  Manage forbidden message.
* Keep raw methods such as scan/NFO enqueue/list/ignore available for internal
  callers when they are already used outside Public Client HTTP routes.
* Replace route-local Manage checks in `http/library.rs` with wrapper calls.
* Remove `http/access.rs` library manage helper if no HTTP route uses it after
  the move.

## Technical Notes

* Relevant code:
  `crates/nako-server/src/http/library.rs`,
  `crates/nako-server/src/http/access.rs`,
  `crates/nako-server/src/app/jobs.rs`,
  `crates/nako-server/src/app/nfo.rs`,
  `crates/nako-server/src/app/library.rs`,
  `crates/nako-server/src/http/tests/library.rs`,
  `crates/nako-server/src/app/tests/nfo.rs`,
  `crates/nako-server/src/app/tests/startup.rs`.
* Relevant specs:
  `.trellis/spec/nako-server/backend/http-api-patterns.md`,
  `.trellis/spec/nako-server/backend/error-handling.md`,
  `.trellis/spec/nako-server/backend/logging-guidelines.md`,
  `.trellis/spec/nako-server/backend/quality-guidelines.md`,
  `.trellis/spec/nako-server/backend/directory-structure.md`.
