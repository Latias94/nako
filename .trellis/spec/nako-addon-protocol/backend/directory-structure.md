# Directory Structure

`nako-addon-protocol` currently keeps the protocol in `src/lib.rs`. Split only
when a contract family becomes too large to review safely in one file.

## Current Layout

- Protocol constants and runtime route inventory.
- Manifest, install descriptor, install guide, runtime, secret-reference, and
  scope types.
- Resource, resource-search, resource-link-check, subtitle, renderer-adapter,
  external-acquisition, task, event, health, artifact, and side-effect payloads.
- `AddonManifestError` and validation helpers.
- Tests covering wire shape, validation, and redaction.

## Module Split Rules

- Keep version and route constants near runtime route inventory.
- Keep wire structs and their validation helpers close enough to review
  together.
- Split by contract family only when the public API remains re-exported from
  `lib.rs`.
- Keep tests next to the contract family they protect.

## Naming Rules

- Use `Addon*` prefixes for public protocol types.
- Use `*Request` and `*Response` for wire envelopes.
- Use `*Declaration` for manifest-declared capabilities.
- Use `*Payload` for protected side-effect payloads.
- Use `*Schema` constants for typed payload schema identifiers.

## Anti-Patterns

- Do not create server-specific protocol modules.
- Do not put official addon catalog builders in this crate.
- Do not hide route path literals in callers when a protocol constant exists.
