# User Playback Profile Preference

## Goal

Persist the current user's default playback profile preference so Public Client
Applications can keep a browser, TV, mobile, or custom capability choice across
sessions and reuse it when preparing playback decisions.

This closes the next backend product gap after profile preset discovery and
server-side profile resolution: users should not have to reconstruct device
capabilities for every client session in a self-hosted media server.

## What I Already Know

* `/playback/profile-presets` exposes safe playback capability templates.
* Playback request boundaries can resolve compact `device_family` and
  `profile_version` inputs into effective playback capabilities.
* `UserPlaybackState` is item progress and watched-state data. It should not be
  widened into a device/profile preference store.
* The Public Client route inventory has `/users/me/playback-state/*` routes but
  no current-user playback profile preference route.
* Existing SQLite and PostgreSQL migrations are registered through version
  `0006`; this feature needs a new schema migration.
* `nako-core` must stay independent from playback/protocol crates, so durable
  repository records should not embed `nako-playback` capability structs.

## Requirements

* Add authenticated current-user JSON routes:
  * `GET /users/me/playback-profile`
  * `PUT /users/me/playback-profile`
  * `DELETE /users/me/playback-profile`
* Scope the preference to the authenticated current principal. Request and
  response bodies must not accept or expose another principal id.
* Store the preference separately from item playback progress state.
* Accept compact request input compatible with playback profile resolution:
  `direct_play`, `device_family`, `profile_version`, and optional explicit
  container/video/audio/HLS override fields.
* Validate and resolve the request through the same playback capability resolver
  used by playback decisions. Persist only the resolved effective capability
  payload, not unresolved user input.
* Return a response containing `preference: null` when no preference exists.
* Return a response containing resolved effective capabilities, `updated_at`,
  and `version` when a preference exists.
* Keep the Public Client protocol permissive and additive: JSON field names stay
  snake_case, unknown future enum variants must not panic, and response bodies
  must not leak local paths, source locators, FFmpeg command details, or runtime
  internals.
* Update SQLite and PostgreSQL adapters, route inventory, OpenAPI inventory, and
  focused tests.

## Acceptance Criteria

* [x] `GET /users/me/playback-profile` returns `{ "preference": null }` for an
      authenticated user with no saved preference.
* [x] `PUT /users/me/playback-profile` with a compact known profile, such as
      `device_family = browser_chromium` and `profile_version = 1`, stores a
      resolved effective profile and returns it.
* [x] A later `GET /users/me/playback-profile` returns the saved resolved
      capabilities with the same current principal scope.
* [x] `DELETE /users/me/playback-profile` clears only the current user's
      preference and reports whether a row was deleted.
* [x] Unknown device families or version mismatches fall back through the same
      safe resolver behavior as playback decision requests.
* [x] Unsupported additive HLS enum variants are rejected at the HTTP boundary
      instead of being silently stored.
* [x] SQLite and PostgreSQL repository contract tests cover absent, upsert,
      replace, and delete behavior.
* [x] Public Client protocol and OpenAPI tests cover route/type visibility.

## Definition of Done

* Code is formatted with `cargo fmt --all`.
* Focused nextest suites pass for affected crates where practical:
  `nako-client-protocol`, `nako-api`, `nako-db`, and `nako-server`.
* A focused `cargo check` passes for the changed backend crates.
* Trellis task validation passes.
* `git diff --check` passes.
* Any durable new convention learned during implementation is recorded in the
  relevant `.trellis/spec/` file.

## Technical Approach

Add a small current-user preference vertical slice:

* Core owns a repository contract and neutral record that stores resolved
  capabilities as JSON text plus `principal_id`, `updated_at_ms`, and `version`.
  This keeps `nako-core` independent from playback/protocol crates.
* DB adapters create a new `user_playback_profile_preferences` table keyed by
  `principal_id`, with an upsert path for replace semantics and a delete path
  for clearing the preference.
* Server HTTP maps the public request DTO into the playback resolver input,
  rejects unsupported additive enum variants, stores the resolved effective
  capability JSON, and maps stored JSON back to Public Client response DTOs.
* Public Client protocol exposes explicit request/response DTOs and adds the
  three current-user routes to the route inventory.
* OpenAPI mirrors the public route and schema shapes.

## Decision (ADR-lite)

**Context**: Nako already has profile preset discovery and per-request playback
capability resolution, but no persisted user default. Extending watched-state
tables would mix unrelated user activity state with client/device preference
state.

**Decision**: Store one resolved effective playback capability preference per
principal behind `/users/me/playback-profile`. Keep it independent from item
playback state and store the durable capability payload as JSON text at the
core repository boundary.

**Consequences**: This MVP is simple and stable for clients, but it is not a
multi-device preference manager. Later work can add named device profiles,
policy merging, and automatic application to playback requests without changing
the current-user route contract.

## Out of Scope

* Multiple saved device profiles per user.
* Automatic User-Agent or hardware detection.
* Automatically applying the saved preference to playback decision or stream
  routes when a request omits capabilities.
* Admin playback policy editing.
* Frontend UI work.
* Addon-provided device profile catalogs.
* Syncing native client local storage.

## Technical Notes

* Relevant architecture maps:
  * `docs/architecture/STATE_ACCESS.md`
  * `docs/architecture/PLAYBACK.md`
* Relevant task lineage:
  * `feat(playback): resolve client profile capabilities`
* Implementation should inspect existing patterns around:
  * `crates/nako-core/src/user_playback.rs`
  * `crates/nako-core/src/repository/user_playback.rs`
  * `crates/nako-db/src/sqlite/user_playback.rs`
  * `crates/nako-db/src/postgres/user_playback.rs`
  * `crates/nako-server/src/http/user_playback.rs`
  * `crates/nako-server/src/http/playback.rs`
  * `crates/nako-client-protocol/src/catalog.rs`
  * `crates/nako-api/src/openapi.rs`
