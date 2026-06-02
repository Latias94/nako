# Directory Structure

`nako` should remain a one-file facade unless the public SDK surface grows
enough to justify feature-specific modules.

## Current Layout

- `src/lib.rs`: crate docs, public re-exports, feature-gated addon client
  re-export, and facade visibility tests.
- `README.md`: consumer examples for `addon_protocol` and optional
  `addon_client`.
- `Cargo.toml`: permissive license, SDK metadata, and feature definitions.

## Module Rules

- Keep root-level re-exports obvious and stable.
- Place real protocol types and validation in `nako-addon-protocol`.
- Place HTTP caller helpers in `nako-addon-client`.
- Add a module only when multiple facade items share real internal structure.
- Keep generated artifacts and server composition code out of this crate.

## Naming Rules

- Use lower_snake_case module aliases that match public consumer imports:
  `addon_protocol` and `addon_client`.
- Feature names should describe optional SDK capabilities, such as
  `addon-client`.
- README examples should use `nako::...` imports, not path dependencies.

## Anti-Patterns

- Do not mirror the whole workspace under `nako::*`.
- Do not expose server, API, database, VFS, catalog, playback, metadata,
  transcode, or runtime supervision crates from this facade.
- Do not add implementation modules just to wrap one re-export.
