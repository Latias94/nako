# Logging Guidelines

`nako-uniffi-bindgen` should not add logging around UniFFI's CLI output.

## Required Patterns

- Let UniFFI print normal CLI diagnostics and errors.
- If wrapper diagnostics are ever added, keep them limited to explicit wrapper
  configuration facts.
- Keep generated artifact paths visible only when the user invoked a generation
  command that already expects path output.

## Forbidden Patterns

- Do not initialize global tracing or logging.
- Do not hide or rewrite UniFFI stderr by default.
- Do not print environment variables, tokens, local server paths, or unrelated
  workspace configuration.
- Do not add noisy build-time logs to normal compile checks.

## Review Checklist

- Is this diagnostic already provided by UniFFI?
- Does the message help a user running the generator?
- Would release automation be a better place for this output?

## Evidence

- `crates/nako-uniffi-bindgen/src/main.rs`
