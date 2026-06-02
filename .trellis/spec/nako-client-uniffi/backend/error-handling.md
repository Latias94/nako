# Error Handling

UniFFI bindings expose core failures as records so foreign runtimes can handle
them without depending on Rust error traits.

## Required Patterns

- Mirror `CoreRuntimeFailureKind` exactly.
- Convert `CoreRuntimeFailure` into UniFFI records without dropping status,
  observed API version, public error, or safe request preview.
- Keep request builder functions returning records/options instead of panicking.
- Preserve `None` for unavailable optional fields.

## Forbidden Patterns

- Do not throw UniFFI exceptions for core request-builder outcomes unless a new
  binding policy is explicitly designed.
- Do not unwrap optional playback targets.
- Do not expose raw access tokens in failures.
- Do not collapse failure kinds into strings.

## Examples

- Missing token during connection probe returns a failure outcome record.
- Unknown playback mode recommendation returns `None`.
- Auth probe public error includes redacted token material.

## Review Checklist

- Does the binding preserve every core failure field?
- Are enum variants mirrored in both directions?
- Does the test cover the exported function?
