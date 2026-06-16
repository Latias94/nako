# feat: Public playback preset capability helpers

## Goal

Make the Public Client playback profile preset catalog directly usable by Rust
client callers. A client should be able to take a
`PlaybackProfilePresetDto` returned by `GET /playback/profile-presets` and turn
it into the explicit flat playback capability request facts used by decision,
remux, HLS, and browser-ticket calls.

## Requirements

* Add client-side conversion helpers from `PlaybackProfilePresetDto` into:
  * `nako-client-core::CorePlaybackCapabilities`
  * `nako-client` playback preset query helpers for decision, remux, and HLS
  * `nako_client_protocol::BrowserPlaybackCapabilitiesDto`
* Preserve the server planning rule: presets are templates only. The server
  must still plan from explicit request facts and must not apply presets
  implicitly.
* Preserve the flat v1 capability field names and wire values.
* Avoid new server, database, storage, transcode, or playback crate
  dependencies in client crates.
* Keep helpers allocation-conscious but simple; cloning preset-owned `String`
  and `Vec<String>` data is acceptable when converting into owned core/client
  request structures.
* Keep unknown additive enum values intact when copying HLS output preferences
  into browser ticket capabilities.
* Add focused tests proving the converted query and browser-ticket body include
  the expected preset facts.

## Acceptance Criteria

* `nako-client-core` exposes a helper that converts a preset DTO into
  `CorePlaybackCapabilities`.
* `nako-client` exposes ergonomic helpers for using a preset with JSON/query
  playback APIs and browser playback tickets.
* Existing `playback_profile_presets()` Rust client method remains unchanged.
* Tests prove a `browser_chromium` preset becomes `device_family`,
  `profile_version`, container/codecs, HDR/subtitle flags, and HLS output
  preferences in outgoing query/body surfaces.
* No Public API, OpenAPI, server route, database, or generated SDK contract
  changes are introduced by this task.

## Out Of Scope

* Automatic preset selection by user agent.
* Server-side implicit preset application.
* Persisted device profile catalogs.
* Admin Web or Public frontend changes.
* TypeScript/Kotlin helper generation.

## Technical Approach

Keep `nako-client-protocol` as the DTO owner and implement conversion at client
adapter boundaries:

* In `nako-client-core`, add a pure conversion helper from
  `nako_client_protocol::PlaybackProfilePresetDto` to
  `CorePlaybackCapabilities`.
* In `nako-client`, add borrowed query/body helper constructors so callers can
  avoid manual comma joining for decision/HLS/remux requests and manual field
  copying for browser-ticket capabilities.
* Tests should use protocol DTO fixtures and existing mock transport/request
  builder assertions.

## Relevant Files

* `crates/nako-client-core/src/playback.rs`
* `crates/nako-client-core/src/lib.rs`
* `crates/nako-client/src/lib.rs`
* `.trellis/spec/nako-client-protocol/backend/index.md`
* `.trellis/spec/nako-client-core/backend/index.md`
* `.trellis/spec/nako-client/backend/index.md`
