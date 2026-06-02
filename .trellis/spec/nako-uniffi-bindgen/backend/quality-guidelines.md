# Quality Guidelines

This crate should remain a minimal, pinned generator entry point. Complexity
belongs in binding crates or release automation.

## Required Patterns

- Keep `uniffi` dependency on the workspace version with `features = ["cli"]`.
- Keep `publish = false`.
- Keep license metadata permissive because this is a tooling crate used around
  public bindings.
- Prefer release scripts or documentation for output directories and language
  targets.
- Use compile checks as the focused validation gate.

## Forbidden Patterns

- Do not duplicate UniFFI's CLI parser.
- Do not generate files during normal `cargo check` or test runs.
- Do not couple the helper to a single language target unless release automation
  requires it.
- Do not import binding target crates just to make the helper "aware" of them.
- Do not add hidden network or filesystem side effects beyond UniFFI's explicit
  generation command.

## Tests Required

- No unit tests are required for the current one-line wrapper.
- Add integration or snapshot-style checks only if custom wrapper behavior is
  introduced.

## Gate Selection

- Focused:
  `cargo check -p nako-uniffi-bindgen`
- Related binding crate:
  `cargo check -p nako-client-uniffi -p nako-uniffi-bindgen --tests`

## Review Checklist

- Is the wrapper still just an invocation boundary?
- Are generated artifacts managed by an explicit workflow?
- Does this change belong in `nako-client-uniffi` instead?
- Does the helper avoid application runtime dependencies?
