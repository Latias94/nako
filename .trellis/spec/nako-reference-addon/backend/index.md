# nako-reference-addon Backend Guidelines

`nako-reference-addon` is a local reference Addon Sidecar fixture. It provides a
minimal Axum router and metadata addon manifest for protocol, client, and server
tests. It is not an official addon catalog and not production sidecar runtime.

## Current Evidence

- `crates/nako-reference-addon/src/lib.rs`
- `crates/nako-reference-addon/Cargo.toml`
- `crates/nako-addon-protocol/src/lib.rs`

## Boundaries

- Provide `reference_manifest` for a minimal metadata Addon Sidecar.
- Provide `build_router` with `/health` and `/metadata` POST routes.
- Provide deterministic demo metadata and NFO/library-file-write payloads.
- Keep fixture behavior simple and protocol-valid.
- Keep production official addon facts in `nako-official-addon-catalog`.

## Executable Contract Summary

1. Scope / Trigger: changes to fixture manifest, routes, demo payloads, or
   protocol response shape update this crate's tests.
2. Signatures: `reference_manifest`, `build_router`, `demo_metadata_patch`, and
   `demo_nfo_export_payload`.
3. Contracts: fixture manifest ID is `nako.reference.metadata`; routes are
   `/health` and `/metadata`; auth is `AddonAuth::None`.
4. Validation & Error Matrix: manifest must pass `validate_manifest`; protected
   write payloads must serialize to protocol shape.
5. Good/Base/Bad Cases: good fixture echoes request ID/addon/resource facts; base
   metadata request falls back to `Unknown Title`; bad cases include invalid
   protocol shape or production-only behavior.
6. Tests Required: manifest validation, entry point/hosted page/schema checks,
   and protected write payload shape checks.
7. Wrong vs Correct: do not grow this into official addon business logic; keep it
   a deterministic protocol fixture.

## Required Patterns

- Keep fixture routes POST-only through Axum `Router`.
- Echo protocol envelope facts in metadata responses.
- Return one `metadata_suggestion` artifact from `/metadata`.
- Keep health response manifest facts aligned with `reference_manifest`.
- Keep demo protected write payloads small and deterministic.

## Forbidden Patterns

- Do not add real provider calls.
- Do not require auth tokens in the reference fixture.
- Do not use this crate as the official addon catalog.
- Do not persist metadata, side effects, or library file writes here.

## Validation

- Focused:
  `cargo nextest run -p nako-reference-addon --no-fail-fast`
- Protocol fixture contract:
  `cargo check -p nako-reference-addon -p nako-addon-protocol --tests`
