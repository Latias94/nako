# Database Guidelines

`nako-uniffi-bindgen` has no database responsibilities. It should not depend on
runtime application crates.

## Required Patterns

- Keep dependencies limited to the UniFFI CLI support needed to run the
  generator.
- Keep database, repository, server, client runtime, and storage crates out of
  this helper binary.
- Treat generated binding outputs as build/release artifacts managed by the
  invoking workflow, not by a database or runtime layer.

## Forbidden Patterns

- Do not depend on `nako-db`, `nako-server`, `nako-api`, `nako-client`,
  `nako-core`, or persistence adapters.
- Do not inspect schema, migrations, catalog records, or runtime configuration.
- Do not generate bindings from database-specific implementation crates unless
  a separate binding crate intentionally exposes those types.
- Do not write generated files into source as a side effect of normal compile.

## Review Checklist

- Does the helper still compile without application runtime dependencies?
- Is any new behavior actually part of a release script instead?
- Are binding source types owned by the binding crate rather than this helper?

## Evidence

- `crates/nako-uniffi-bindgen/Cargo.toml`
- `crates/nako-uniffi-bindgen/src/main.rs`
