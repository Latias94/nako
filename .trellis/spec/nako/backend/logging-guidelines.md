# Logging Guidelines

`nako` currently emits no logs. That is the preferred default for a public SDK
facade.

## Required Patterns

- Keep diagnostics in the crates that perform real work.
- If facade-level diagnostics become necessary, use the workspace tracing
  conventions and keep messages about feature/export setup only.
- Keep README examples free of logging setup unless the underlying SDK crate
  requires it.

## Forbidden Patterns

- Do not log from re-export-only code.
- Do not log tokens, shared secrets, URLs with credentials, local server paths,
  source locators, or addon request bodies.
- Do not initialize a global logger from this crate.
- Do not add noisy startup or import-time diagnostics.

## Review Checklist

- Does the log communicate a facade-level event rather than underlying SDK
  behavior?
- Should the diagnostic live in `nako-addon-client` instead?
- Is the message safe for third-party integration logs?

## Evidence

- `crates/nako/src/lib.rs`
