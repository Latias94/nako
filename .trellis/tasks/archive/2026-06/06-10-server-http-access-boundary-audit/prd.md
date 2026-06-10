# refactor: audit remaining server HTTP access boundaries

## Goal

Continue the server access-boundary cleanup by auditing remaining
`nako-server` HTTP route-local access checks, then selecting one focused slice
where moving authorization into an app service improves consistency without
changing public API behavior.

## What I already know

* Recent slices moved metadata item access, renderer transport use access, and
  playback session control access from HTTP helpers into app-service contracts.
* `crates/nako-server/src/http/access.rs` still provides route-local
  `RequiredLibraryAccess`, `has_library_access`, and `require_library_access`
  helpers.
* Code inspection found business Library Access helper use is concentrated in
  `crates/nako-server/src/http/library.rs`; `metadata.rs` only uses the
  route-local administrator guard.
* Public Library browse/read routes cover `GET /libraries`,
  `GET /libraries/{library_id}`, `GET /libraries/{library_id}/sources`, and
  `GET /libraries/{library_id}/items`.
* Manage-command routes in the same file fan out to library scan, NFO, and
  ingestion-failure services; they are a larger follow-up slice.
* The next slice should be small enough to verify with focused `nextest` and
  `cargo check -p nako-server --tests`.

## Assumptions (temporary)

* Public Library browse/read access belongs in `LibraryAppService`, while HTTP
  should parse query/path inputs and return DTOs.
* The task should preserve public status codes and error body semantics unless
  a current route behavior is demonstrably inconsistent with an existing spec.
* Raw/internal app-service methods may remain where they serve non-public
  runtime flows; Public Client route authorization should live at app-service
  boundaries when there are or may be non-HTTP callers.

## Open Questions

* None for the MVP; code inspection selects Public Library browse/read surfaces
  as the focused slice.

## Requirements (evolving)

* Inventory remaining `http::access` helper usage in `nako-server`.
* Move Public Library browse/read access enforcement from `http/library.rs` into
  `LibraryAppService`.
* Keep `GET /libraries` filtering semantics: ordinary principals see only
  browse-visible libraries and the returned page count matches the filtered
  response.
* Preserve `GET /libraries/{library_id}` and
  `GET /libraries/{library_id}/sources` browse denial behavior.
* Preserve `GET /libraries/{library_id}/items` hidden-library behavior:
  inaccessible libraries return `NakoError::NotFound { entity: "library" }`.
* Preserve existing public API DTOs, status codes, and redaction behavior.
* Add or update focused tests for any moved access decision.

## Acceptance Criteria (evolving)

* [x] Remaining route-local access helper use sites are inventoried.
* [x] Public Library browse/read slice is selected with rationale and
      Manage-command routes are explicitly out of scope.
* [x] HTTP library browse/read handlers pass the authenticated principal to
      `LibraryAppService` instead of calling route-local Library Access helpers.
* [x] `LibraryAppService` enforces browse access before returning library,
      source, or item read responses and preserves current hidden-library
      semantics where they exist.
* [x] Regression tests cover the moved access decision or cleanup behavior.
* [x] Focused `cargo nextest` and `cargo check -p nako-server --tests` pass.

## Definition of Done

* [x] Rust code is formatted with `cargo fmt --all` when Rust changes are made.
* [x] Focused server tests for the selected slice pass.
* [x] `cargo check -p nako-server --tests` passes when Rust changes are made.
* [x] `git diff --check` passes.
* [x] Relevant Trellis spec or task notes are updated.
* [ ] Task is archived, journal is recorded, commits are pushed.

## Out of Scope (temporary)

* Broad user/role access model redesign.
* Schema migrations or public API shape changes.
* Changing Admin route guard behavior.
* Moving Manage-command access for scan, NFO import/export, or ingestion
  failure commands in this slice.
* Large multi-module rewrites without a dedicated follow-up task.

## Technical Approach

* Add Public Library browse/read methods to `LibraryAppService` that accept
  `&AuthenticatedPrincipal`.
* Keep raw or lower-level library methods available only if existing internal
  callers need them.
* Replace route-local `has_library_access` / `require_library_access` calls in
  read handlers with app-service calls.
* Keep Manage-command routes on the existing HTTP helper until a dedicated
  follow-up can move scan/NFO/ingestion command authorization into their
  owning app services.

## Technical Notes

* Likely files to inspect:
  `crates/nako-server/src/http/access.rs`,
  `crates/nako-server/src/http/library.rs`,
  `crates/nako-server/src/app/library.rs`,
  related HTTP/app tests.
* Inventory result:
  `http::access::{has_library_access, require_library_access}` is only used by
  `http/library.rs`; `http::access::require_administrator` is used by
  `http/metadata.rs`.
* Relevant specs:
  `.trellis/spec/nako-server/backend/http-api-patterns.md`,
  `.trellis/spec/nako-server/backend/error-handling.md`,
  `.trellis/spec/nako-server/backend/quality-guidelines.md`.
