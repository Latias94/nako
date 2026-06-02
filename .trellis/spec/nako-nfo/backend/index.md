# nako-nfo Backend Guidelines

`nako-nfo` owns NFO codec, import/export preview, sidecar URI planning, and
round-trip behavior. It works through VFS and repository traits instead of raw
filesystem or database access.

## Current Evidence

- `crates/nako-nfo/src/codec.rs`
- `crates/nako-nfo/src/import.rs`
- `crates/nako-nfo/src/export.rs`
- `crates/nako-nfo/src/preview.rs`
- `crates/nako-nfo/src/workflow.rs`
- `crates/nako-nfo/src/summary.rs`

## Boundaries

- Parse and render NFO XML through `NfoCodec`.
- Use `MovieNfoCodec` for current movie-style fixtures and behavior.
- Use `NfoService` for storage-backed import/export operations.
- Use `StorageBackend` for sidecar reads and writes.
- Keep canonical metadata ownership in core repositories and metadata workflows.

## Required Patterns

- Preserve unknown XML fields and comments when `render_preserving` is used.
- Surface invalid XML as `NakoError::InvalidInput`.
- Convert source locators to sidecar `.nfo` URIs through existing helpers.
- Use preview decisions before mutating sidecars or imported data.
- Track content fingerprints for import/export decisions.

## Forbidden Patterns

- Do not bypass VFS for local file paths.
- Do not overwrite unknown NFO fields during preserving renders.
- Do not silently import sidecar data into canonical metadata without policy.
- Do not treat default preservation as supported unless the codec implements it.

## Validation

- Focused:
  `cargo nextest run -p nako-nfo --no-fail-fast`
- Cross-layer:
  `cargo check -p nako-nfo -p nako-vfs -p nako-core --tests`
