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
   48 paths; streaming routes are explicitly marked `StreamingBuilder`.
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

## Forbidden Patterns

- Do not depend on `nako-core`, `nako-api`, `nako-server`, database, or reqwest.
- Do not expose server-only fields such as principal ID, raw file locators,
  output paths, bearer tokens, or transcode internals.
- Do not change public wire strings without tests.
- Do not add a route without updating `PUBLIC_CLIENT_ROUTES`.

## Validation

- Focused:
  `cargo nextest run -p nako-client-protocol --no-fail-fast`
- Public client contract:
  `cargo check -p nako-client-protocol -p nako-client-core -p nako-client --tests`
