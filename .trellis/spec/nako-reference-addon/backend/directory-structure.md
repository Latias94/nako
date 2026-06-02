# Directory Structure

`nako-reference-addon` is intentionally small and currently lives in
`src/lib.rs`.

## Current Layout

- `REFERENCE_ADDON_ID`: fixture manifest ID.
- `reference_manifest`: protocol-valid metadata addon manifest.
- `build_router`: Axum router with `/health` and `/metadata`.
- `demo_metadata_patch`: deterministic metadata patch payload.
- `demo_nfo_export_payload`: deterministic library file write payload.
- Private route handlers and focused tests.

## Module Rules

- Keep the fixture in one file while it remains minimal.
- Split routes into a private module only if more protocol fixture endpoints are
  added.
- Keep tests next to fixture behavior.
- Keep protocol types imported from `nako-addon-protocol`.

## Naming Rules

- Use `reference_*` for fixture manifest/router concepts.
- Use `demo_*` for deterministic payload helpers.
- Use `REFERENCE_ADDON_ID` for the manifest ID.

## Anti-Patterns

- Do not create official addon modules here.
- Do not create provider-specific fixture logic.
- Do not add database or storage modules.
