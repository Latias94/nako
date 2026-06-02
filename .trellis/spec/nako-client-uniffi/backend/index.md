# nako-client-uniffi Backend Guidelines

`nako-client-uniffi` exposes the transport-neutral `nako-client-core` builders
and connection probe state machine through UniFFI records, enums, and exported
functions. It does not execute network calls.

## Current Evidence

- `crates/nako-client-uniffi/src/lib.rs`
- `crates/nako-client-uniffi/Cargo.toml`
- `crates/nako-client-core/src/lib.rs`

## Boundaries

- Define UniFFI-safe mirrors of core records and enums.
- Export request builder functions for connection probe, browse, artwork,
  playback, HLS, and user playback.
- Convert between UniFFI mirror types and `nako-client-core` types.
- Call `uniffi::setup_scaffolding!()`.
- Keep reqwest transport, async SDK calls, and CLI behavior outside this crate.

## Executable Contract Summary

1. Scope / Trigger: new core builder exported to mobile/foreign clients, new
   core input/output type, or conversion behavior updates this crate.
2. Signatures: UniFFI `Record`/`Enum` mirrors and exported functions such as
   `start_connection_probe`, `build_playback_decision_request`, and browse/user
   playback builders.
3. Contracts: exported functions delegate to `nako-client-core` and preserve
   safe previews, request IDs, URLs, headers, and optional preflight requests.
4. Validation & Error Matrix: runtime failures are returned as
   `CoreRuntimeFailure` records, not thrown exceptions.
5. Good/Base/Bad Cases: good bindings match core output exactly; bad bindings
   duplicate URL logic or omit redacted safe previews.
6. Tests Required: exported surface tests for connection probe, playback target,
   browse, artwork, HLS, and user playback builders.
7. Wrong vs Correct: do not implement route logic in UniFFI; convert to core
   inputs and delegate.

## Required Patterns

- Mirror core types with `uniffi::Record` and `uniffi::Enum`.
- Add `From` conversions in both directions when callers pass input types.
- Keep exported functions synchronous request builders.
- Preserve token redaction in `safe_preview`.
- Keep crate type as `cdylib` and `rlib`.

## Forbidden Patterns

- Do not depend on `nako-client`, reqwest, tokio, server, API, database, or
  storage crates.
- Do not perform network IO.
- Do not expose raw access tokens in safe previews.
- Do not hand-roll path/query encoding here.

## Validation

- Focused:
  `cargo nextest run -p nako-client-uniffi --no-fail-fast`
- Binding compile:
  `cargo check -p nako-client-core -p nako-client-uniffi --tests`
