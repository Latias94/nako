# nako Backend Guidelines

`nako` is the public Rust SDK facade for addon and protocol integration. It is
intentionally small: it re-exports public protocol crates and optional SDK
helpers without exposing server internals.

## Pre-Development Checklist

- Read [Directory Structure](./directory-structure.md) before adding facade
  modules, re-exports, features, or README examples.
- Read [Database Guidelines](./database-guidelines.md) before exposing any type
  that might pull persistence or server implementation details into this crate.
- Read [Error Handling](./error-handling.md) before adding fallible facade
  helpers.
- Read [Quality Guidelines](./quality-guidelines.md) before changing public
  exports, feature flags, crate metadata, or license-facing behavior.
- Read [Logging Guidelines](./logging-guidelines.md) before adding diagnostics.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Public SDK facade layout and feature-gated exports | Filled from code |
| [Database Guidelines](./database-guidelines.md) | Persistence isolation for the public facade | Filled from code |
| [Error Handling](./error-handling.md) | Fallible facade helper boundaries | Filled from code |
| [Quality Guidelines](./quality-guidelines.md) | Re-export stability, licensing, and feature rules | Filled from code |
| [Logging Guidelines](./logging-guidelines.md) | No-diagnostics facade default | Filled from code |

## Authority / Evidence

- `crates/nako/src/lib.rs`
- `crates/nako/README.md`
- `crates/nako/Cargo.toml`
- `crates/nako-addon-protocol/src/lib.rs`
- `crates/nako-addon-client/src/lib.rs`

## Boundaries

- Re-export `nako-addon-protocol` as `nako::addon_protocol`.
- Re-export `nako-addon-client` as `nako::addon_client` only behind the
  `addon-client` feature.
- Keep server, API, database, catalog, playback, metadata, storage, and runtime
  internals outside this crate.
- Preserve the permissive Apache-2.0 OR MIT license surface for addon and
  integration authors.

## Executable Contract Summary

1. Scope / Trigger: public SDK facade exports, feature flags, README examples,
   or crate metadata changes update this crate.
2. Signatures: `nako::addon_protocol` is always available;
   `nako::addon_client` is feature-gated by `addon-client`.
3. Contracts: this crate delegates behavior to underlying public crates and
   does not define server implementation logic.
4. Validation & Error Matrix: fallible behavior stays in the re-exported crates;
   facade tests assert visibility and version wiring.
5. Good/Base/Bad Cases: good facade exports public SDK crates only; bad facade
   exports server persistence, runtime, or internal API structs.
6. Tests Required: add visibility or feature-gate tests when public re-exports
   change.
7. Wrong vs Correct: do not add helper implementations here when the behavior
   belongs in `nako-addon-protocol`, `nako-addon-client`, or a server crate.

## Validation

- Focused:
  `cargo nextest run -p nako --no-fail-fast`
- Feature surface:
  `cargo check -p nako --features addon-client --tests`
