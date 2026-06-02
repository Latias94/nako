# Directory Structure

`nako-uniffi-bindgen` is intentionally a single binary with no library target.

## Current Layout

- `src/main.rs`: delegates directly to `uniffi::uniffi_bindgen_main()`.
- `Cargo.toml`: depends on workspace `uniffi` with the `cli` feature and sets
  `publish = false`.

## Module Rules

- Keep the wrapper in `src/main.rs`.
- Do not add a `lib.rs` unless workspace tooling needs reusable generator
  functions.
- Do not place generated language bindings in this crate.
- Keep binding record/enum definitions in the crate being bound, currently
  `nako-client-uniffi`.

## Naming Rules

- Keep the package and binary name `nako-uniffi-bindgen`.
- Keep command behavior aligned with upstream UniFFI CLI naming.
- Use crate names, not generated language names, when documenting binding
  targets.

## Anti-Patterns

- Do not add hand-written CLI parsing around UniFFI unless a concrete release
  workflow requires it.
- Do not import `nako-client`, server, API, database, or runtime crates.
- Do not use this crate as a general build script or release packager.
