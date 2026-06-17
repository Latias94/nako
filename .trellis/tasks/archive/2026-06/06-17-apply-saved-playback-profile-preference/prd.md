# Apply Saved Playback Profile Preference

## Goal

Make the previously persisted current-user playback profile preference affect
playback behavior. When a Public Client caller does not send explicit playback
capability facts, playback decision and direct/remux/HLS playback startup should
use the authenticated principal's saved resolved capability profile. This turns
`/users/me/playback-profile` from stored settings into part of the self-hosted
playback product loop.

## What I Already Know

* `GET|PUT|DELETE /users/me/playback-profile` now persists one resolved
  capability payload per authenticated principal.
* Current playback decision and stream routes still resolve empty query input
  to `ClientPlaybackCapabilities::default()`.
* `PlaybackCapabilitiesQuery` already models every explicit flat capability
  query field.
* Playback preference records store neutral JSON in `nako-core` so `nako-server`
  must parse stored JSON into `ClientPlaybackCapabilities`.
* Explicit per-request capability fields must continue to override saved
  defaults; saved preferences are only the fallback when the route receives no
  explicit playback capability facts.

## Requirements

* Apply a saved playback profile preference to authenticated Public Client
  playback decision requests when no capability query fields are present.
* Apply the same fallback to authenticated direct stream, direct stream
  preflight, remux stream/preflight, and HLS playlist startup when no capability
  query fields are present and the request is not already bound to a browser or
  renderer playback session.
* Preserve explicit capability query behavior. Any supplied capability query
  field means the request resolves from the query and does not merge with the
  saved preference.
* Preserve browser ticket and renderer transport session behavior. Tickets and
  existing playback sessions already carry their own client context and should
  not be re-planned from saved preferences.
* Preserve existing default behavior when no saved preference exists.
* Treat invalid stored preference JSON as a server-side data error rather than
  silently falling back to defaults.
* Keep this slice server-side. Do not add new public fields, new routes, new DB
  migrations, client SDK methods, frontend UI, or automatic User-Agent
  detection.

## Acceptance Criteria

* [ ] A playback decision with no capability query fields uses the current
      user's saved Chromium profile and chooses Transcode for an HEVC source
      that default capabilities would Direct Play.
* [ ] A playback decision with an explicit capability query field still uses the
      request query and can Direct Play the same HEVC source when
      `video_codec=hevc` is supplied.
* [ ] A playback decision with no saved preference preserves the existing
      default capability behavior.
* [ ] Direct stream and HEAD preflight without explicit capability query use the
      saved preference before starting a playback session.
* [ ] Remux and HLS startup without explicit capability query use the saved
      preference when creating a new session, while ticket/session-bound routes
      preserve existing behavior.
* [ ] Invalid stored playback capability JSON returns an error and does not
      create a misleading default-planned session.
* [ ] Focused server tests cover the fallback and explicit override semantics.

## Definition of Done

* `cargo fmt --all` passes.
* `cargo check -p nako-server --tests` passes.
* Focused `cargo nextest` for affected playback route/service tests passes.
* `git diff --check` passes.
* Trellis task validation passes.
* Durable behavior notes are recorded in `.trellis/spec/` or `docs/api` if the
  implementation establishes a new server contract.

## Technical Approach

Add a server-side capability source helper:

* `PlaybackCapabilitiesQuery` should expose whether any capability field was
  explicitly provided.
* HTTP playback handlers should ask `PlaybackAppService` to resolve current
  principal default capabilities only when query capabilities are absent.
* `PlaybackAppService` should read
  `UserPlaybackProfilePreferenceRepository::get_user_playback_profile_preference`
  through the existing runtime store boundary, parse `capabilities_json` into
  `ClientPlaybackCapabilities`, and fall back to default capabilities when the
  row is absent.
* Existing query resolution remains authoritative when any capability field is
  present.

## Decision (ADR-lite)

**Context**: Persisting preferences without applying them leaves users and
clients rebuilding capability input for every playback request.

**Decision**: Apply saved profile preferences as a server-side fallback only
when playback routes receive no explicit capability facts. Explicit request
facts remain stronger than saved defaults.

**Consequences**: This closes the first user-profile playback loop without a
wire-contract change. Later work can add named device profiles or explicit
`profile_id` selection without changing this fallback rule.

## Out of Scope

* Multiple saved device profiles per user.
* Frontend settings UI.
* Public API schema changes.
* Client SDK changes beyond existing calls.
* Applying saved preferences to browser ticket bodies that already include an
  explicit `capabilities` object.
* User-Agent or runtime hardware auto-detection.
* Admin policy merging or per-library playback policy editing.

## Technical Notes

* Relevant specs:
  * `.trellis/spec/nako-server/backend/http-api-patterns.md`
  * `.trellis/spec/nako-server/backend/quality-guidelines.md`
  * `.trellis/spec/nako-playback/backend/quality-guidelines.md`
  * `.trellis/spec/nako-db/backend/database-guidelines.md`
  * `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
* Relevant code:
  * `crates/nako-server/src/http/playback.rs`
  * `crates/nako-server/src/app/playback/mod.rs`
  * `crates/nako-server/src/http/tests/playback.rs`
  * `crates/nako-server/src/app/tests/playback.rs`
  * `crates/nako-server/src/app/user_playback.rs`
  * `crates/nako-core/src/repository/user_playback.rs`
* Relevant previous commit:
  * `d463075c feat(playback): persist user profile preferences`
