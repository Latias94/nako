# nako-client-protocol Backend Guidelines

`nako-client-protocol` owns the Public Client API wire contract: route inventory,
version headers, account DTOs, browse/catalog DTOs, playback DTOs, renderer DTOs,
user playback state, and playlist DTOs. It must remain dependency-light and
independent from server internals.

## Current Evidence

- `crates/nako-client-protocol/src/lib.rs`
- `crates/nako-client-protocol/src/catalog.rs`
- `crates/nako-client-protocol/Cargo.toml`
- `CONTEXT.md`

## Boundaries

- Define `CLIENT_PROTOCOL_VERSION`, `API_VERSION_HEADER`, and
  `PLAYBACK_SESSION_ID_HEADER`.
- Maintain `PUBLIC_CLIENT_ROUTES` and exposure classification for JSON methods
  versus streaming builders.
- Define public DTOs only; do not map from server/domain types here.
- Preserve additive string compatibility through `Other(String)` wire enums.
- Keep transport, request building, and reqwest execution outside this crate.

## Executable Contract Summary

1. Scope / Trigger: route, DTO, header, public enum, error code, browse,
   playback, renderer, user playback, or playlist shape changes update this
   crate.
2. Signatures: public route inventory, `HealthResponse`, `ErrorResponse`,
   `PageInfo`, catalog DTOs, playback DTOs, renderer DTOs, and user state DTOs.
3. Contracts: current public API version is `v1`; route inventory currently has
   52 paths; streaming routes are explicitly marked `StreamingBuilder`.
4. Validation & Error Matrix: client-visible errors use `ClientErrorCode` string
   values; unknown additive wire strings decode to `Other(String)` where the
   enum uses `public_string_value!`.
5. Good/Base/Bad Cases: good DTOs hide source locators, bearer tokens, server
   paths, principal IDs, and raw transcode output paths; bad DTOs leak internal
   server state into public clients.
6. Tests Required: route inventory, serde shape, unknown string preservation,
   sensitive field absence, and error code round-trip tests.
7. Wrong vs Correct: do not expose `nako-core` IDs or server structs directly;
   define explicit public DTOs and map in API/server layers.

## Required Patterns

- Use serde DTOs with explicit public fields.
- Use `public_string_value!` for additive wire enums that must tolerate future
  server values.
- Keep `PageInfo { limit, offset, returned }` as the public pagination envelope.
- Keep public playback URLs safe and ticketed; do not expose source locators.
- Keep current-user routes under `/users/me`.
- Keep provider governance, Metadata Candidate Review, batch apply,
  idempotency, raw provider payload, and related hierarchy application route
  fragments out of `PUBLIC_CLIENT_ROUTES`; these are Admin API surfaces unless a
  future PRD explicitly changes the Public Client contract.
- Keep Public Client playback capabilities on the flat v1 field set until a
  dedicated profile contract task changes it:
  `direct_play`, `container`, `video_codec`, `audio_codec`,
  `max_video_bitrate`, `max_width`, `max_height`, `max_audio_channels`,
  `supports_hdr`, `supports_subtitles`, `hls_variant_policy`, and
  `hls_segment_container`. Remux query/browser ticket remux planning may also
  carry `output_container`.
- `playback_profile_id` is a current-user named-profile selector query for new
  playback decision, Direct Stream, Remux, and HLS playlist startup requests.
  It is not a capability DTO field, not a preset id, and must stay out of
  browser-ticket bodies, renderer bodies, existing session routes, and HLS
  segment routes.
- Keep playback profile preset discovery under the authenticated JSON route
  `/playback/profile-presets`. The DTO is a catalog template, not a playback
  request: expose `family`, `device_family`, `profile_version`, flat capability
  fields, and HLS output preferences, and keep runtime/operator facts out.
- Keep current-user playback profile preference under
  `GET|PUT|DELETE /users/me/playback-profile`. The PUT request is a compact
  default-profile facade body with plural capability-set fields (`containers`,
  `video_codecs`, `audio_codecs`); the response returns a resolved
  `ClientPlaybackCapabilitiesDto` plus `updated_at` and `version`. The response
  must not expose principal ids, raw request JSON, local paths, source
  locators, FFmpeg/runtime facts, or operator policy.
- Keep current-user named playback profile CRUD under
  `GET|POST /users/me/playback-profiles` and
  `GET|PUT|DELETE /users/me/playback-profiles/{profile_id}`. Named profile
  request bodies use a required `name` on create, optional `name` on update,
  optional `is_default`, and flattened plural capability-set fields. Named
  profile responses return `profile_id`, `name`, resolved `capabilities`,
  `is_default`, `updated_at`, and `version`, plus `PageInfo` for list.

## Scenario: Current-User Playback Profile Preference Contract

### 1. Scope / Trigger

- Trigger: adding or changing the Public Client current-user playback profile
  preference route, request/response DTOs, route inventory entry, generated
  OpenAPI/SDK surface, or Rust client methods.
- Scope:
  `SetUserPlaybackProfilePreferenceRequest`,
  `UserPlaybackProfilePreferenceResponse`,
  `UserPlaybackProfilePreferenceDto`,
  `DeleteUserPlaybackProfilePreferenceResponse`,
  `PUBLIC_CLIENT_ROUTES`, `nako-api` OpenAPI/SDK generators,
  `nako-client-core`, and `nako-client`.

### 2. Signatures

- Routes:
  - `GET /users/me/playback-profile -> UserPlaybackProfilePreferenceResponse`
  - `PUT /users/me/playback-profile + SetUserPlaybackProfilePreferenceRequest
    -> UserPlaybackProfilePreferenceResponse`
  - `DELETE /users/me/playback-profile ->
    DeleteUserPlaybackProfilePreferenceResponse`
- Request fields:
  `direct_play`, `device_family`, `profile_version`, `containers`,
  `video_codecs`, `audio_codecs`, `max_video_bitrate`, `max_width`,
  `max_height`, `max_audio_channels`, `supports_hdr`,
  `supports_subtitles`, `hls_variant_policy`, and
  `hls_segment_container`.
- Response:
  `UserPlaybackProfilePreferenceResponse { preference:
  Option<UserPlaybackProfilePreferenceDto> }`, where the DTO contains
  `capabilities: ClientPlaybackCapabilitiesDto`, `updated_at`, and `version`.

### 3. Contracts

- The route is authenticated and always current-user scoped through `/users/me`;
  request and response bodies must not accept or expose `principal_id`.
- PUT is a preference write, not a playback decision request. The server
  resolves the compact request through playback profile resolution before
  storage and returns the resolved effective capability DTO.
- Request capability-set fields are plural (`containers`, `video_codecs`,
  `audio_codecs`) because the body can store a whole effective preference.
  Browser playback tickets and query parameters may still use singular
  request-preference field names where already specified.
- `preference: null` is the no-row response; do not synthesize defaults in the
  protocol DTO.
- Additive HLS enums may deserialize as `Other(String)` in the protocol crate,
  but the HTTP boundary rejects unsupported values before persistence.
- The response capability DTO remains client/player facts only. It must not
  include source locators, local paths, bearer tokens, FFmpeg command/runtime
  facts, hardware probe facts, operator policy, or raw transcode internals.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| No stored preference for the current principal | Return `{ "preference": null }` |
| PUT uses known current `device_family` and `profile_version` | Store and return resolved preset capabilities |
| PUT includes explicit capability overrides | Store and return resolved capabilities with those overrides |
| PUT uses unknown device family or mismatched profile version | Preserve safe identity/version and fall back through resolver defaults |
| PUT includes `hls_variant_policy` or `hls_segment_container` as `Other` | Server returns invalid input and does not store the row |
| Response contains `principal_id`, source locator, path, token, FFmpeg, runtime, or policy facts | Contract violation |
| Route is added without OpenAPI/SDK/client-core/client updates | Contract drift |

### 5. Good/Base/Bad Cases

- Good: a browser client reads `/playback/profile-presets`, sends
  `{ "device_family": "browser_chromium", "profile_version": 1 }` to
  `/users/me/playback-profile`, and receives resolved Chromium capabilities.
- Good: a TV app sends explicit `containers`, `video_codecs`, and HLS output
  preferences when its local player differs from a preset.
- Base: older clients never call this route and continue sending explicit
  per-request capabilities to playback decision/stream routes.
- Bad: storing unresolved request JSON and re-resolving it differently after a
  future preset change.
- Bad: exposing another user's preference through a route with a path
  `principal_id` or `user_id`.

### 6. Tests Required

- Protocol route inventory and serde-shape tests prove `/users/me` routes,
  compact PUT body fields, and resolved response fields.
- API OpenAPI/SDK package-entry tests prove the generated TypeScript and
  Kotlin SDKs expose GET/PUT/DELETE and the DTOs.
- Rust client and client-core request builder tests prove method, path, auth,
  request IDs, and JSON body behavior.
- Server route tests prove absent/read/write/delete behavior and rejection of
  unsupported additive HLS enum values.

### 7. Wrong vs Correct

#### Wrong

```rust
pub struct SetUserPlaybackProfilePreferenceRequest {
    pub principal_id: String,
    pub container: Option<String>,
}
```

#### Correct

```rust
pub struct SetUserPlaybackProfilePreferenceRequest {
    pub containers: Option<Vec<String>>,
    pub device_family: Option<String>,
    pub profile_version: Option<u32>,
}
```

Current-user routes derive the principal from authentication and store an
effective capability set, not a one-off playback query.

## Scenario: Current-User Named Playback Profile Contract

### 1. Scope / Trigger

- Trigger: adding or changing the Public Client current-user named playback
  profile CRUD route, DTOs, route inventory entry, generated OpenAPI/SDK
  surface, or Rust client methods.
- Scope:
  `UserPlaybackProfilesResponse`, `UserPlaybackProfileResponse`,
  `UserPlaybackProfileDto`, `CreateUserPlaybackProfileRequest`,
  `UpdateUserPlaybackProfileRequest`,
  `UserPlaybackProfileCapabilitiesRequest`,
  `DeleteUserPlaybackProfileResponse`, `PUBLIC_CLIENT_ROUTES`, `nako-api`
  OpenAPI/SDK generators, `nako-client-core`, and `nako-client`.

### 2. Signatures

- Routes:
  - `GET /users/me/playback-profiles -> UserPlaybackProfilesResponse`
  - `POST /users/me/playback-profiles + CreateUserPlaybackProfileRequest ->
    UserPlaybackProfileResponse`
  - `GET /users/me/playback-profiles/{profile_id} ->
    UserPlaybackProfileResponse`
  - `PUT /users/me/playback-profiles/{profile_id} +
    UpdateUserPlaybackProfileRequest -> UserPlaybackProfileResponse`
  - `DELETE /users/me/playback-profiles/{profile_id} ->
    DeleteUserPlaybackProfileResponse`
- Create request fields:
  `name`, optional `is_default`, and flattened
  `UserPlaybackProfileCapabilitiesRequest`.
- Update request fields:
  optional `name`, optional `is_default`, and flattened
  `UserPlaybackProfileCapabilitiesRequest`.
- Capability request fields:
  `direct_play`, `device_family`, `profile_version`, `containers`,
  `video_codecs`, `audio_codecs`, `max_video_bitrate`, `max_width`,
  `max_height`, `max_audio_channels`, `supports_hdr`,
  `supports_subtitles`, `hls_variant_policy`, and
  `hls_segment_container`.
- Response DTO:
  `UserPlaybackProfileDto { profile_id, name, capabilities, is_default,
  updated_at, version }`.

### 3. Contracts

- Routes are authenticated and always current-user scoped through `/users/me`;
  request and response bodies must not accept or expose `principal_id` or
  `user_id`.
- Named profiles are the productized source-of-truth profile contract. The
  single `/users/me/playback-profile` route is a compatibility facade for the
  current user's default named profile.
- The collection route is bounded and returns `PageInfo`; list response bodies
  must not synthesize profiles outside the authenticated principal.
- `profile_id` is an opaque server-owned path segment and must be percent
  encoded by clients/builders when inserted into URLs.
- Create requires `name`; update may leave `name`, `is_default`, and capability
  fields absent so callers can replace only the fields they intend to change.
- Capability-set fields are plural (`containers`, `video_codecs`,
  `audio_codecs`) because stored profiles represent resolved effective
  capability sets.
- Additive HLS enums may deserialize as `Other(String)` in the protocol crate,
  but the HTTP boundary rejects unsupported values before persistence.
- Response capability DTOs remain client/player facts only. They must not
  include source locators, local paths, bearer tokens, FFmpeg command/runtime
  facts, hardware probe facts, operator policy, or raw transcode internals.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| No profiles exist for the current principal | Return an empty `profiles` array and `PageInfo` with `returned = 0` |
| Create includes supported preset identity and capability overrides | Store resolved effective capabilities and return one `profile` DTO |
| Create or update sets `is_default = true` | Server makes that profile the only default for the current principal |
| Delete targets an existing profile | Return `deleted: true` with the same `profile_id` |
| Delete targets a missing profile | Return a deterministic not-found or idempotent delete response as defined by the server route task; do not leak other principals |
| Request contains `principal_id`, `user_id`, source locator, local path, token, FFmpeg, runtime, hardware, or policy facts | Contract violation |
| Route is added without OpenAPI/SDK/client-core/client updates | Contract drift |

### 5. Good/Base/Bad Cases

- Good: a TV app creates `Living Room TV` with plural codec/container sets,
  marks it default, and receives a resolved capability DTO plus version facts.
- Good: a browser lists `/users/me/playback-profiles?limit=20&offset=0` and
  receives only the authenticated principal's bounded profile page.
- Base: older clients keep using `/users/me/playback-profile` and observe the
  default named profile through the compatibility facade.
- Bad: exposing another user's profile through
  `/users/{principal_id}/playback-profiles`.
- Bad: storing unresolved request JSON and returning it as the response
  capability DTO.

### 6. Tests Required

- Protocol route inventory tests prove both named profile paths exist, have
  JSON SDK exposure, and carry `GET|POST` / `GET|PUT|DELETE` methods.
- Protocol serde-shape tests prove request bodies use plural capability-set
  fields and response bodies omit principal/user identifiers.
- API OpenAPI/SDK package-entry tests prove TypeScript and Kotlin generated
  artifacts expose list/create/get/update/delete methods and DTOs.
- Rust client-core tests prove request IDs, auth, page query, JSON body
  behavior, and percent encoding of `{profile_id}`.
- Rust client tests prove async methods use current-user paths and percent
  encoded profile IDs.
- Server route tests prove current-user scoping, bounded list, create, read,
  update, delete, default selection, and unsupported additive HLS enum
  rejection.

### 7. Wrong vs Correct

#### Wrong

```rust
pub struct UserPlaybackProfileDto {
    pub principal_id: String,
    pub capabilities_json: String,
}
```

#### Correct

```rust
pub struct UserPlaybackProfileDto {
    pub profile_id: String,
    pub name: String,
    pub capabilities: ClientPlaybackCapabilitiesDto,
    pub is_default: bool,
    pub updated_at: String,
    pub version: u64,
}
```

Public Client profile management exposes an opaque current-user profile
contract. Internal principal ownership and persisted JSON stay behind the API
boundary.

## Forbidden Patterns

- Do not depend on `nako-core`, `nako-api`, `nako-server`, database, or reqwest.
- Do not expose server-only fields such as principal ID, raw file locators,
  output paths, bearer tokens, or transcode internals.
- Do not change public wire strings without tests.
- Do not add a route without updating `PUBLIC_CLIENT_ROUTES`.
- Do not add Candidate Review, Provider Mapping governance, or related
  hierarchy plan/apply route shapes to the Public Client inventory.

## Validation

- Focused:
  `cargo nextest run -p nako-client-protocol --no-fail-fast`
- Playback capability field set:
  `cargo nextest run -p nako-client-protocol public_playback_capability_dtos_keep_current_flat_field_contract --no-fail-fast`
- Public client contract:
  `cargo check -p nako-client-protocol -p nako-client-core -p nako-client --tests`
