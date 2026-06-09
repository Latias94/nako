# refactor: move user playback access into app service

## Goal

Move Public Client user playback state access checks from
`http::user_playback` into `UserPlaybackAppService`, so playback state reads and
writes own their Library Access invariants in the app service instead of
depending on route-local `require_item_access` / `require_source_access` guards.

## What I already know

* `http::user_playback` currently performs route-local access checks before:
  * `GET /users/me/playback-state/items/{item_id}` with Browse item access
  * `PUT /users/me/playback-state/items/{item_id}/progress` with Play item
    access and optional Play source access
  * `PUT /users/me/playback-state/items/{item_id}/watched` with Play item
    access and optional Play source access
* `UserPlaybackAppService` already owns item existence, source-to-item
  validation, stale progress handling, watched policy, and continue-watching
  projections.
* `list_continue_watching_entries(&AuthenticatedPrincipal, page)` already uses
  access-aware repository projection and is not the target of this cleanup.
* `require_item_access` and `require_source_access` remain used by metadata,
  playback byte/plan routes, renderer routes, and selected artwork routes; this
  task should not delete them globally.

## Assumptions

* Public route paths, request DTOs, and response DTOs stay unchanged.
* Existing user playback behavior remains:
  * Browse access is enough to read a default/current user playback state.
  * Play access is required to update progress or watched state.
  * Optional `source_id` must be playable and must belong to the item.
* Missing item/source and source-from-another-item public errors remain
  compatible with existing tests.
* No schema, migration, repository trait in `nako-core`, or client contract
  changes are needed.

## Requirements

* Make user playback app-service read/write requests principal-aware enough to
  enforce item/source access internally.
* Enforce Browse access inside `UserPlaybackAppService::get_state`.
* Enforce Play access inside `UserPlaybackAppService::update_progress` and
  `set_watched_state` before committing state writes.
* Preserve source-to-item validation after source existence/access is resolved.
* Keep `http::user_playback` handlers thin: parse path/body, parse optional
  source/timestamps, pass principal to app service, return JSON.
* Remove `require_item_access` / `require_source_access` imports from
  `http::user_playback` if no handler still needs them.
* Add focused tests proving hidden/browse-only user playback writes are
  rejected by the app service and route layer, and visible reads/writes still
  work.

## Acceptance Criteria

* [x] `http::user_playback` no longer imports or calls `require_item_access` or
  `require_source_access`.
* [x] `UserPlaybackAppService` enforces Browse for get-state and Play for
  progress/watched mutations.
* [x] Source access and source-to-item validation remain compatible with
  existing public error behavior.
* [x] Focused app/route tests cover access rejection and visible success.
* [x] Focused server checks and `git diff --check` pass.

## Verification

* `cargo fmt --all`
* `cargo check -p nako-server --tests`
* `cargo nextest run -p nako-server user_playback --no-fail-fast`
* `git diff --check`

## Definition of Done

* Tests added/updated where behavior is observable.
* HTTP user playback code is thinner and has no playback-state access
  compensation.
* Relevant `.trellis/spec/` entry is updated if this establishes a reusable app
  service access pattern.
* Trellis task is archived and session journal is updated after commit.

## Out of Scope

* Refactoring playback byte routes, playback planning/admission, renderer
  transport, metadata manage access, or selected artwork access.
* Changing playback policy evaluation or transcoding admission.
* Changing Public Client DTOs, route paths, schemas, or repository trait shape
  outside the server app-store facade.
* Changing continue-watching pagination/projection semantics.

## Technical Notes

* Relevant specs:
  * `.trellis/spec/nako-server/backend/http-api-patterns.md`
  * `.trellis/spec/nako-server/backend/database-guidelines.md`
  * `.trellis/spec/nako-server/backend/error-handling.md`
  * `.trellis/spec/nako-server/backend/quality-guidelines.md`
  * `.trellis/spec/guides/cross-layer-thinking-guide.md`
  * `.trellis/spec/guides/code-reuse-thinking-guide.md`
* Relevant architecture map: `docs/architecture/STATE_ACCESS.md`.
* Relevant code:
  * `crates/nako-server/src/http/user_playback.rs`
  * `crates/nako-server/src/app/user_playback.rs`
  * `crates/nako-server/src/http/tests/user_playback.rs`
  * `crates/nako-server/src/app/tests/user_playback.rs`
