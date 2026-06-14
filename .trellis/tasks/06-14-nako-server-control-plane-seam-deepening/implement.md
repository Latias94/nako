# Nako server control-plane seam deepening

## Plan

1. Read the current startup/composition/runtime code path and identify the
   narrowest workflow extraction.
2. Add or update focused tests around startup report and runtime diagnostics
   behavior before changing the seam.
3. Extract the startup workflow behind a named app-service entry point while
   keeping `NakoApp` as the composition root.
4. Tighten any runtime-supervisor coupling that remains obviously shallow.
5. Run focused server formatting, tests, and check commands.

## Validation

* `cargo fmt --all`
* `cargo nextest run -p nako-server --no-fail-fast`
* `cargo check -p nako-server --tests`

## Review Gates

* Confirm startup report content is still redaction-safe.
* Confirm runtime diagnostics still surface the same useful state.
* Confirm no HTTP route shape or public DTO changed.

## Rollback

If the first extraction makes the seam less clear, stop after the smallest
behavior-preserving change and keep the remaining startup/runtime tightening as
a follow-on task.

