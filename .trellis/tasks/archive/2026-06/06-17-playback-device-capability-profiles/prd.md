# feat: Playback device capability profiles

## Goal

Add the first Public Client contract slice for named playback capability
profiles so Client Applications can send a stable device/profile identity
alongside the existing flat v1 capability fields. This should make playback
decisions easier to reproduce, cache, debug, and evolve toward browser,
Android, renderer, and future TV clients without breaking current callers.

## Requirements

* Keep every existing flat v1 playback capability field compatible:
  `direct_play`, `container`, `video_codec`, `audio_codec`,
  `max_video_bitrate`, `max_width`, `max_height`, `max_audio_channels`,
  `supports_hdr`, `supports_subtitles`, `hls_variant_policy`, and
  `hls_segment_container`.
* Add an additive profile identity to Public Client capability DTOs and query
  surfaces. The profile identity must describe client/player facts only, not
  host runtime, FFmpeg, hardware, resource pressure, or operator policy.
* Thread the profile identity through protocol DTOs, OpenAPI, SDK generation,
  Rust client builders, server query/body mappings, renderer mapping, browser
  ticket mapping, and HTTP API docs.
* Include the profile identity in playback target profile identity keys when it
  is present, because it can change reproducibility and cache/debug grouping
  even when flat fields are identical.
* Treat the profile identity as optional. Old clients that omit it must produce
  the same playback decisions and request keys as before.
* Keep public outputs redaction-safe: no Source Locators, paths, playback
  tickets, bearer tokens, FFmpeg commands, GPU device names, or server-only
  diagnostics.

## Acceptance Criteria

* [x] `nako-client-protocol` exposes optional profile fields on
      `BrowserPlaybackCapabilitiesDto` and `ClientPlaybackCapabilitiesDto`.
* [x] Public OpenAPI query/body schemas and generated TypeScript/Kotlin SDK
      surfaces include the new profile fields.
* [x] `nako-client` and `nako-client-core` can render the profile fields for
      playback decision, remux, HLS, and browser-ticket related surfaces where
      they already render flat capability fields.
* [x] `nako-server` maps playback decision queries, remux/HLS queries, browser
      ticket bodies, and renderer media capability bodies into
      `ClientPlaybackCapabilities` without losing the profile identity.
* [x] `nako-playback` profile identity includes the optional profile facts when
      present and remains unchanged when absent.
* [x] Contract tests prove the new fields stay out of forbidden host/runtime
      facts and that existing flat-field compatibility tests remain green.
* [x] HTTP docs mention the additive profile identity and keep the flat v1
      compatibility baseline explicit.

## Definition of Done

* Focused Rust tests pass for touched protocol, API, client, client-core,
  playback, and server surfaces.
* `cargo fmt --all` or targeted Rust formatting is run.
* `git diff --check` passes.
* Trellis task validates and is updated with evidence.
* Commit uses a Conventional Commit message.

## Technical Approach

Use two optional public fields, matching the existing API contract guidance for
future additive profile data:

* `device_family`: a stable lowercase profile name such as `browser_chromium`,
  `android_media3`, or `tv_webos`.
* `profile_version`: an optional unsigned integer controlled by the Client
  Application.

These fields are intentionally descriptive and client-owned. They do not
replace the flat capability facts; the planner still decides from explicit
container, codec, HDR, subtitle, bitrate, resolution, and HLS fields.

## Decision (ADR-lite)

**Context**: Nako already has stable playback decision reasons and flat v1
capability fields, but full Jellyfin-style device profiles are too broad for
one slice.

**Decision**: Add a minimal named profile identity as additive metadata on the
existing capability contract. Do not add a server-owned device profile catalog
or implicit compatibility table in this task.

**Consequences**: Clients can start reporting stable profile identity now, while
future tasks can add server-known profile catalogs or richer client capability
reporting without breaking v1 flat-field callers.

## Implementation Summary

* Added optional `device_family` and `profile_version` fields to playback
  capability records, protocol DTOs, query/body mappings, Rust clients, UniFFI,
  OpenAPI, generated TypeScript/Kotlin SDKs, and HTTP API docs.
* Included normalized profile identity facts in
  `PlaybackTargetProfile::identity()` while preserving old request identity
  behavior when the fields are omitted.
* Kept the fields descriptive and client-owned; no Admin diagnostics, FFmpeg,
  hardware, resource pressure, or host runtime facts were added to Public
  Client DTOs.

## Spec Sync

No new code-spec section was required. The existing
`.trellis/spec/nako-api/backend/quality-guidelines.md` scenario "Public Client
Playback Capability Contract Parity" already covers this change class,
including protocol DTOs, OpenAPI/SDK outputs, server mapping, Rust clients,
profile identity, generated SDKs, and HTTP docs.

## Verification Evidence

* `rustfmt --edition 2024` on the touched Rust files passed.
* `cargo fmt --all -- --check` passed.
* `cargo check -p nako-playback -p nako-client-protocol -p nako-api -p nako-client-core -p nako-client -p nako-client-uniffi -p nako-client-cli -p nako-server --tests` passed.
* `cargo check -p nako-api --examples` passed.
* `cargo nextest run -p nako-playback playback_target_profile_identity --no-fail-fast` passed.
* `cargo nextest run -p nako-client-protocol public_playback_capability_dtos_keep_current_flat_field_contract public_playback_decision_uses_protocol_owned_types --no-fail-fast` passed.
* `cargo nextest run -p nako-api public_openapi_playback public_sdk_playback --no-fail-fast` passed.
* `cargo nextest run -p nako-server playback_capability_queries_map_all_current_flat_fields browser_playback_ticket_capabilities_map_all_current_flat_fields renderer_media_capabilities_map_all_current_flat_fields --no-fail-fast` passed.
* `cargo nextest run -p nako-client-core playback --no-fail-fast` passed.
* `cargo nextest run -p nako-client streaming_request_builders_use_stable_paths_methods_headers_and_queries --no-fail-fast` passed.
* `cargo nextest run -p nako-client-uniffi uniffi_surface_preserves_full_playback_capability_query_fields --no-fail-fast` passed.
* `git diff --check` passed.

## Out of Scope

* Server-owned Jellyfin/Plex-style device profile database.
* Automatic browser feature detection.
* Admin Web playback diagnostics.
* Android UI implementation beyond generated SDK/contract surfaces.
* Changing decision selection rules based solely on the profile name.
* Schema migrations.

## Technical Notes

* Read before implementation:
  * `CONTEXT.md`
  * `.trellis/spec/nako-playback/backend/index.md`
  * `.trellis/spec/nako-client-protocol/backend/index.md`
  * `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
  * `.trellis/spec/nako-server/backend/http-api-patterns.md`
  * `docs/architecture/PLAYBACK.md`
* Likely code areas:
  * `crates/nako-playback/src/lib.rs`
  * `crates/nako-playback/src/capability.rs`
  * `crates/nako-client-protocol/src/catalog.rs`
  * `crates/nako-client-protocol/src/lib.rs`
  * `crates/nako-api/src/public_client.rs`
  * `crates/nako-api/src/openapi.rs`
  * `crates/nako-api/src/sdk.rs`
  * `crates/nako-client-core/src/playback.rs`
  * `crates/nako-client/src/lib.rs`
  * `crates/nako-server/src/http/playback.rs`
  * `crates/nako-server/src/http/renderer.rs`
  * `docs/api/HTTP_API.md`
