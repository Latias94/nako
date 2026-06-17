# nako-client-core Backend Guidelines

`nako-client-core` owns transport-neutral Public Client request builders,
connection probing, response interpretation, percent encoding, and redaction.
It performs no network IO and has no runtime dependency on reqwest.

## Current Evidence

- `crates/nako-client-core/src/lib.rs`
- `crates/nako-client-core/src/request.rs`
- `crates/nako-client-core/src/response.rs`
- `crates/nako-client-core/src/connection.rs`
- `crates/nako-client-core/src/browse.rs`
- `crates/nako-client-core/src/playback.rs`

## Boundaries

- Build `CoreHttpRequest` and `CoreSafeRequestPreview`.
- Add bearer auth headers from access tokens when requested.
- Encode path segments and query parameters.
- Build browse, artwork, playback, HLS, and user playback requests.
- Interpret HTTP/API-version failures into `CoreRuntimeFailure`.
- Keep actual transport in `nako-client` or foreign-language clients.

## Executable Contract Summary

1. Scope / Trigger: any route builder, request ID, query parameter, auth header,
   response interpretation, connection probe, or redaction change updates this
   crate.
2. Signatures: `CoreHttpRequestSpec`, `build_core_request`,
   `start_connection_probe`, `advance_connection_probe`, route builders, and
   `interpret_core_response`.
3. Contracts: request builders use stable request IDs, percent-encoded path
   segments, optional query params, Authorization headers, and safe previews.
4. Validation & Error Matrix: missing token, non-2xx HTTP, unsupported API
   version, invalid health JSON, and unknown probe response IDs map to
   `CoreRuntimeFailureKind`.
5. Good/Base/Bad Cases: good builders redact tokens in previews; base streaming
   builders produce unauthenticated direct/remux/HLS URLs; bad builders expose
   secrets or skip path encoding.
6. Tests Required: path/query encoding, auth redaction, connection probe state,
   response interpretation, browse/artwork/playback/user playback builders.
7. Wrong vs Correct: do not execute reqwest here; return request facts so callers
   can transport them.

## Required Patterns

- Use `CoreHttpRequestSpec` and `build_core_request` for new request builders.
- Keep request IDs in `ids.rs` when they are reused across bindings.
- Use `encode_path_segment` for all user/domain IDs in paths.
- Keep `safe_preview` populated and redacted.
- Use `API_VERSION_HEADER` and `CLIENT_PROTOCOL_VERSION` for response checks.
- Playback profile preset helpers must convert public protocol DTOs into owned
  `CorePlaybackCapabilities` without adding server/playback dependencies.
  Unknown additive HLS policy/container wire values are omitted from core
  capability fields because core enums only model known request facts.
- Explicit playback profile selection must use `CorePlaybackSelection` and
  encode `playback_profile_id` on playback decision, Direct Stream, Remux, and
  HLS playlist builders. Keep the selector separate from
  `CorePlaybackCapabilities`; it is a current-user profile choice, not a player
  capability fact.
- Current-user playback profile preference builders must use the stable JSON
  route `/users/me/playback-profile` with request IDs
  `user_playback.profile_preference`,
  `user_playback.profile_preference.set`, and
  `user_playback.profile_preference.delete`. They require bearer auth, do not
  add query parameters, and leave the PUT body as caller-owned JSON.
- Current-user named playback profile builders must use the stable JSON routes
  `/users/me/playback-profiles` and
  `/users/me/playback-profiles/{profile_id}` with request IDs
  `user_playback.profiles`, `user_playback.profiles.create`,
  `user_playback.profiles.get`, `user_playback.profiles.update`, and
  `user_playback.profiles.delete`. They require bearer auth, percent-encode
  `profile_id`, apply only `limit`/`offset` on list, and leave POST/PUT bodies
  as caller-owned JSON.

## Forbidden Patterns

- Do not depend on reqwest, tokio, server, database, or storage crates.
- Do not perform IO.
- Do not put raw access tokens in safe previews or public failures.
- Do not use lossy path concatenation without percent encoding.

## Validation

- Focused:
  `cargo nextest run -p nako-client-core --no-fail-fast`
- Binding/SDK contract:
  `cargo check -p nako-client-core -p nako-client -p nako-client-uniffi --tests`
