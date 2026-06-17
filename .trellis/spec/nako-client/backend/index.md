# nako-client Backend Guidelines

`nako-client` is the async Rust Public Client SDK. It wraps
`nako-client-protocol` DTOs and `nako-client-core` builders with a mockable
transport and a reqwest implementation.

## Current Evidence

- `crates/nako-client/src/lib.rs`
- `crates/nako-client/Cargo.toml`
- `crates/nako-client-core/src/lib.rs`
- `crates/nako-client-protocol/src/lib.rs`

## Boundaries

- Provide `NakoClient` async JSON methods.
- Provide streaming request builders returning `ClientRequest` without sending.
- Provide `ClientTransport` and `ReqwestTransport`.
- Convert core requests/responses to SDK request/response types.
- Map core runtime failures to `NakoClientError`.
- Keep CLI and UniFFI wrappers in their own crates.

## Executable Contract Summary

1. Scope / Trigger: new Public Client method, streaming request builder, query
   type, transport behavior, or SDK error mapping updates this crate.
2. Signatures: `NakoClient`, `ClientTransport`, `ReqwestTransport`,
   `ClientRequest`, `ClientResponse`, query structs, and `NakoClientError`.
3. Contracts: JSON methods send requests and decode DTOs; streaming builders
   produce request facts; API version header must match `API_VERSION`.
4. Validation & Error Matrix: invalid base URL/path/header, transport error,
   encode/decode error, API HTTP error, invalid version header, unsupported API
   version, missing access token, and invalid core response map to
   `NakoClientError`.
5. Good/Base/Bad Cases: good SDK calls use mockable transport; base health and
   login do not require auth; bad calls expose tokens or bypass version checks.
6. Tests Required: mock transport requests, API error mapping, version checks,
   JSON encode/decode, streaming builder facts, and Cargo dependency boundaries.
7. Wrong vs Correct: do not duplicate URL logic in SDK methods; use core builders
   where the route is shared with CLI/UniFFI.

## Required Patterns

- Normalize base URL to end with `/` and strip query/fragment.
- Use `ClientTransport` for testable transport behavior.
- Use protocol DTOs re-exported from `nako-client`.
- Add `Authorization` only for methods requiring auth.
- Validate `x-nako-api-version` on responses.
- For authenticated Public Client JSON discovery routes, add the async
  `NakoClient` method and keep the matching transport-neutral request builder
  in `nako-client-core` when CLI/UniFFI bindings need the same route facts.
- Playback profile preset convenience helpers must expand discovered presets
  into explicit capability query/body fields. They must not send a preset ID or
  require the server to apply an implicit preset. Browser ticket capability
  helpers preserve additive HLS policy/container enum strings from
  `nako-client-protocol`.
- Current-user playback profile preference methods must expose
  `get_user_playback_profile_preference`,
  `set_user_playback_profile_preference`, and
  `delete_user_playback_profile_preference` against
  `/users/me/playback-profile`. Re-export the request, response, preference
  DTO, and `ClientPlaybackCapabilitiesDto` types from `nako-client-protocol`
  so Rust SDK callers do not need a second dependency for the method result.
- Current-user named playback profile methods must expose
  `list_user_playback_profiles`, `create_user_playback_profile`,
  `get_user_playback_profile`, `update_user_playback_profile`, and
  `delete_user_playback_profile` against `/users/me/playback-profiles` and
  `/users/me/playback-profiles/{profile_id}`. Re-export the named profile
  request/response DTOs from `nako-client-protocol`, require auth on all
  methods, preserve optional list pagination, and percent-encode `profile_id`
  before inserting it into URLs.

## Forbidden Patterns

- Do not depend on `nako-server`, `nako-api`, database, storage, streaming, or
  transcode crates.
- Do not log or expose bearer tokens.
- Do not send streaming requests in methods that are documented as builders.
- Do not bypass `ClientTransport` in tests.

## Validation

- Focused:
  `cargo nextest run -p nako-client --no-fail-fast`
- Full client stack:
  `cargo nextest run -p nako-client-protocol -p nako-client-core -p nako-client --no-fail-fast`
