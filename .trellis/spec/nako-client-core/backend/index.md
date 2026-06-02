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
