# refactor: audit playback session control access boundary

## Goal

Move Public Client playback session control authorization for inspect, cancel,
and heartbeat from HTTP route-local code into `PlaybackAppService`, while
preserving the existing session-hiding behavior for non-owners and principals
without current source `Play` access.

## What I already know

* Previous access-boundary slices moved subtitle, renderer transport, and
  metadata item access checks into app services.
* `crates/nako-server/src/http/playback.rs` still calls
  `require_playback_session_control_access` before get/cancel/heartbeat
  session responses.
* The route-local helper intentionally returns `playback_session` `NotFound`
  when the session belongs to another principal, the source is gone, or the
  principal no longer has library `Play` access.
* `PlaybackAppService` already owns session persistence, heartbeat mutation,
  cancellation, source loading, and playback policy/access helpers.

## Assumptions

* Session-control hiding semantics are a Public Client app-service contract,
  not an HTTP route concern.
* Internal app-service helpers such as raw `get_playback_session` and
  `cancel_playback_session` may remain available for runtime/internal flows.
* HTTP routes should only parse request data, pass the authenticated principal,
  map public heartbeat state, and shape DTOs.

## Requirements

* Add app-service wrappers for Public Client playback session control:
  inspect, cancel, and heartbeat.
* The wrappers must verify session ownership and current source `Play` access
  before exposing or mutating the session.
* Preserve existing public error semantics:
  wrong owner, missing source, revoked source `Play` access, and missing
  session all return `NakoError::NotFound` for `playback_session`.
* Preserve existing terminal-session conflict behavior for authorized
  cancellation and heartbeat attempts.
* Remove route-local `require_playback_session_control_access` if it becomes
  unused.
* Keep media-byte stream authorization, browser playback tickets, renderer
  transport, and internal runtime cancellation behavior out of scope.

## Acceptance Criteria

* [x] HTTP playback session get/cancel/heartbeat routes call app-service
      control wrappers instead of route-local session access helpers.
* [x] App-service tests prove wrong-owner and revoked `Play` access are hidden
      as `playback_session` `NotFound`.
* [x] Existing HTTP session-control route tests continue to prove public
      `404` behavior for non-owner and revoked access.
* [x] `require_playback_session_control_access` is removed if unused.
* [x] Focused playback tests and `cargo check -p nako-server --tests` pass.

## Definition of Done

* [x] Rust code is formatted with `cargo fmt --all`.
* [x] Focused `cargo nextest run -p nako-server playback_session --no-fail-fast`
      or a narrower equivalent passes.
* [x] `cargo check -p nako-server --tests` passes.
* [x] `git diff --check` passes.
* [x] Task is archived, journal is recorded, commits are pushed.

## Out of Scope

* Changing direct/remux/HLS byte route access behavior.
* Changing browser playback ticket issuance or validation.
* Changing renderer transport access.
* Changing terminal session state-machine semantics.

## Technical Notes

* Relevant code paths:
  `crates/nako-server/src/http/playback.rs`,
  `crates/nako-server/src/app/playback/mod.rs`,
  `crates/nako-server/src/http/tests/playback.rs`,
  `crates/nako-server/src/app/tests/playback.rs`.
* Relevant specs:
  `.trellis/spec/nako-server/backend/http-api-patterns.md`,
  `.trellis/spec/nako-server/backend/error-handling.md`,
  `.trellis/spec/nako-server/backend/quality-guidelines.md`.
