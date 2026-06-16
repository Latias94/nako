# feat: Public playback profile preset discovery

## Goal

Expose Nako's built-in playback profile preset catalog through a read-only
Public Client API so Client Applications can discover recommended flat
capability templates before requesting playback. This should make browser,
mobile, TV, cast, and renderer clients easier to configure while preserving
the rule that actual playback planning uses explicit request facts.

## What I Already Know

* `nako-playback` now owns `PlaybackProfilePreset` and
  `playback_profile_presets()`.
* Admin playback runtime diagnostics already exposes the preset catalog for
  operators through `profile_presets`.
* `nako-client-protocol` owns Public Client wire DTOs and `PUBLIC_CLIENT_ROUTES`.
* `nako-api` generates Public OpenAPI, TypeScript SDK, and Kotlin SDK artifacts
  from Public Client contract code.
* The existing flat capability fields remain the planner authority:
  `direct_play`, containers, video/audio codecs, bitrate/resolution/channel
  limits, HDR/subtitle support, and HLS output preferences.

## Requirements

* Add a Public Client read-only endpoint for playback profile presets.
* Define protocol-owned Public DTOs for preset discovery. The response must
  expose only safe client capability facts: profile family, `device_family`,
  `profile_version`, flat capability fields, and HLS output preferences.
* Keep the endpoint authenticated like normal Public Client read APIs; do not
  create a public unauthenticated route.
* Add the route to `PUBLIC_CLIENT_ROUTES`, OpenAPI output, generated TypeScript
  SDK, Kotlin SDK, Rust client-core/client request builders, and HTTP docs.
* Server mapping must read from `nako-playback::playback_profile_presets()` and
  return the protocol DTO. No database, user-agent detection, runtime hardware
  probing, or session inspection.
* Preserve planner behavior: clients may choose to copy a preset into playback
  requests, but the server must not apply presets implicitly.
* Keep Admin-only diagnostics, hardware facts, FFmpeg facts, resource pressure,
  local paths, Source Locators, bearer tokens, playback tickets, and operator
  policy out of the Public response.

## Acceptance Criteria

* [ ] Public Client route inventory includes the preset discovery route and
      marks it as a normal JSON API route.
* [ ] Public DTO serialization tests prove known presets are emitted with safe
      flat capability fields and no `unknown` preset.
* [ ] Server route test proves the endpoint returns the catalog and remains
      redaction-safe.
* [ ] OpenAPI, generated TypeScript SDK, Kotlin SDK, Rust client-core, and
      Rust client surfaces can call the route.
* [ ] Existing playback decision behavior is unchanged.
* [ ] HTTP API docs describe the route and explicitly say presets are client
      request templates, not implicit server rules.

## Definition of Done

* Focused protocol/API/server/client tests pass.
* Generated artifacts are refreshed from source.
* Formatting and whitespace checks pass.
* Trellis task validates and records verification evidence.
* Commit uses a Conventional Commit message and is pushed.

## Technical Approach

Add Public Client DTOs in `nako-client-protocol` and map
`PlaybackProfilePreset` into those DTOs in `nako-api`. Add a route such as
`GET /playback/profile-presets` to the Public Client route inventory, server
route module, OpenAPI, SDK generators, and clients.

The response shape should mirror the safe flat capability fields already used
by `ClientPlaybackCapabilitiesDto`, but keep it as a distinct preset DTO so
future fields can be versioned without confusing a request body with a catalog
entry.

## Decision (ADR-lite)

**Context**: Admin diagnostics can preview built-in presets, but client
applications still need to hard-code or guess the same capability templates.
Pushing this into public discovery reduces duplicated client logic.

**Decision**: Add a read-only authenticated Public Client discovery route backed
by the pure playback preset catalog. Do not add automatic server-side preset
application.

**Consequences**: Client Applications can bootstrap capability reporting from
server-known templates. Future richer device profile work can evolve through
explicit Public Client DTO/version changes.

## Out of Scope

* Applying presets automatically to playback decisions.
* User-agent/browser detection.
* Server-owned per-device database or editable profile catalog.
* Admin Web UI changes.
* Database migrations.
* Hardware/FFmpeg/operator runtime fields in Public Client DTOs.

## Technical Notes

* Relevant specs/docs:
  * `CONTEXT.md`
  * `docs/architecture/PLAYBACK.md`
  * `.trellis/spec/nako-client-protocol/backend/index.md`
  * `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
  * `.trellis/spec/nako-api/backend/quality-guidelines.md`
  * `.trellis/spec/nako-server/backend/http-api-patterns.md`
* Likely code areas:
  * `crates/nako-client-protocol/src/catalog.rs`
  * `crates/nako-client-protocol/src/lib.rs`
  * `crates/nako-api/src/public_client.rs`
  * `crates/nako-api/src/openapi.rs`
  * `crates/nako-api/src/sdk.rs`
  * `crates/nako-client-core`
  * `crates/nako-client`
  * `crates/nako-server/src/http/playback.rs` or a nearby Public route module
  * `docs/api/HTTP_API.md`
