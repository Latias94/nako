# refactor: move user playlist item access into app service

## Goal

Move user playlist item mutation browse-access checks from
`http::user_playlist` into `UserPlaylistAppService`, so playlist item add,
remove, and reorder operations own their item visibility invariants in the app
service instead of relying on route-local `require_item_access` guards.

## What I already know

* `http::user_playlist` currently calls `require_item_access(... Browse)` before:
  * `PUT /users/me/playlists/{playlist_id}/items/{item_id}`
  * `DELETE /users/me/playlists/{playlist_id}/items/{item_id}`
  * `PUT /users/me/playlists/{playlist_id}/items/reorder`
* `UserPlaylistAppService` already owns playlist existence, ownership,
  optimistic version, duplicate handling, and projection reads.
* `UserPlaylistAppService` already accepts `AuthenticatedPrincipal` for read
  projections, but mutation request structs currently carry only
  `UserPrincipalId`.
* `require_item_access` is still used by metadata and user playback routes; this
  task should not delete or globally replace it.

## Assumptions

* Public playlist route shape and DTO fields stay unchanged.
* Existing behavior remains: ordinary principals can add/remove/reorder only
  items they can browse.
* Playlist ownership and expected-version behavior stay unchanged.
* Missing or stale playlist/item behavior should remain compatible with current
  public error envelopes.
* No schema, migration, repository trait, or client contract changes are needed.

## Requirements

* Make playlist item mutation app-service requests principal-aware enough to
  enforce item browse access internally.
* Enforce browse access inside `UserPlaylistAppService::add_item`,
  `remove_item`, and `reorder_items` before committing the mutation.
* Keep `http::user_playlist` handlers thin: parse request/path, pass principal
  to app service, return JSON.
* Remove `require_item_access` from `http::user_playlist` if no playlist handler
  still needs it.
* Preserve `require_item_access` for metadata/user playback routes.
* Add focused tests proving hidden item add/remove/reorder are forbidden and
  visible playlist mutations still work.

## Acceptance Criteria

* [x] `http::user_playlist` no longer imports or calls `require_item_access`.
* [x] `UserPlaylistAppService` enforces browse access for item add/remove/reorder.
* [x] Existing playlist ownership/version behavior remains unchanged.
* [x] Focused route/app tests cover hidden item mutation rejection and visible
  mutation success.
* [x] Focused server checks and `git diff --check` pass.

## Verification

* `cargo fmt --all`
* `cargo check -p nako-server --tests`
* `cargo nextest run -p nako-server user_playlist --no-fail-fast`
* `git diff --check`

## Definition of Done

* Tests added/updated where behavior is observable.
* HTTP route code is thinner and has no playlist item access compensation.
* Trellis task is archived and session journal is updated after commit.

## Out of Scope

* Refactoring metadata, user playback, playback, renderer, or selected artwork
  access checks.
* Changing Public Client DTOs or route paths.
* Adding repository access projection methods or database migrations.
* Changing playlist sharing/collaboration semantics.

## Technical Notes

* Relevant specs:
  * `.trellis/spec/nako-server/backend/http-api-patterns.md`
  * `.trellis/spec/nako-server/backend/database-guidelines.md`
  * `.trellis/spec/nako-server/backend/quality-guidelines.md`
  * `.trellis/spec/guides/cross-layer-thinking-guide.md`
  * `.trellis/spec/guides/code-reuse-thinking-guide.md`
* Relevant code:
  * `crates/nako-server/src/http/user_playlist.rs`
  * `crates/nako-server/src/app/user_playlist.rs`
  * `crates/nako-server/src/http/tests/user_playlist.rs`
  * `crates/nako-server/src/app/tests/user_playlist.rs`
* Architecture map: `docs/architecture/STATE_ACCESS.md`.
