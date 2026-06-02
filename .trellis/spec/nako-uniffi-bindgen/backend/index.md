# nako-uniffi-bindgen Backend Guidelines

`nako-uniffi-bindgen` is a tiny helper binary that delegates to UniFFI's CLI
entry point. It exists so workspace tooling can invoke a pinned, workspace
configured binding generator.

## Pre-Development Checklist

- Read [Directory Structure](./directory-structure.md) before changing binary
  layout, generated artifact policy, or helper entry points.
- Read [Database Guidelines](./database-guidelines.md) before adding any
  runtime crate dependencies.
- Read [Error Handling](./error-handling.md) before wrapping or replacing the
  UniFFI CLI entry point.
- Read [Quality Guidelines](./quality-guidelines.md) before changing dependency
  features, publication settings, or validation commands.
- Read [Logging Guidelines](./logging-guidelines.md) before adding diagnostics.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | One-binary UniFFI CLI wrapper layout | Filled from code |
| [Database Guidelines](./database-guidelines.md) | No runtime or persistence dependencies | Filled from code |
| [Error Handling](./error-handling.md) | Delegate CLI error handling to UniFFI | Filled from code |
| [Quality Guidelines](./quality-guidelines.md) | Pinned CLI feature and generated artifact policy | Filled from code |
| [Logging Guidelines](./logging-guidelines.md) | No extra logging around UniFFI CLI | Filled from code |

## Authority / Evidence

- `crates/nako-uniffi-bindgen/src/main.rs`
- `crates/nako-uniffi-bindgen/Cargo.toml`
- `crates/nako-client-uniffi/src/lib.rs`

## Boundaries

- Own only the local `nako-uniffi-bindgen` binary.
- Delegate command-line behavior to `uniffi::uniffi_bindgen_main()`.
- Keep UniFFI binding type definitions in binding crates such as
  `nako-client-uniffi`.
- Keep generated binding artifacts out of this crate unless release automation
  explicitly decides otherwise.

## Executable Contract Summary

1. Scope / Trigger: binding generation helper binary, UniFFI CLI feature, or
   workspace generator invocation changes update this crate.
2. Signatures: `main` delegates to `uniffi::uniffi_bindgen_main()`.
3. Contracts: the binary uses the workspace `uniffi` dependency with the `cli`
   feature and is not published.
4. Validation & Error Matrix: CLI argument parsing and failures are handled by
   UniFFI, not by custom wrapper code.
5. Good/Base/Bad Cases: good wrapper remains one line of delegation; bad wrapper
   duplicates UniFFI CLI parsing or imports application crates.
6. Tests Required: compile checks are enough unless wrapper behavior is added.
7. Wrong vs Correct: do not add binding definitions here; update
   `nako-client-uniffi` or another binding crate.

## Validation

- Focused:
  `cargo check -p nako-uniffi-bindgen`
- Binding surface:
  `cargo check -p nako-client-uniffi -p nako-uniffi-bindgen --tests`
