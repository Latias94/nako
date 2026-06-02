# Quality Guidelines

Protocol changes must protect compatibility, redaction, and explicit addon
permission boundaries.

## Required Patterns

- Add serde wire-shape tests for every new public payload or enum.
- Add validation tests for each new manifest declaration rule.
- Add redaction tests when a payload can carry credentials, URLs, tokens,
  passwords, local locators, or renderer tickets.
- Keep `as_str` implementations aligned with serde snake-case values.
- Keep install-guide output redaction-safe and operator-oriented.

## Forbidden Patterns

- Do not introduce implicit server behavior through protocol defaults.
- Do not allow breaking changes under the same protocol version.
- Do not add a public enum variant without tests for serialization and matching
  client/server behavior.
- Do not trust Addon Hosted Pages with Nako admin credentials.

## Tests Required

- Manifest good/base/bad validation tests.
- Runtime route inventory tests.
- Resource, task, event, and health envelope validation tests.
- Addon install descriptor and guide tests.
- Redaction tests for custom `Debug` implementations.

## Gate Selection

- Focused:
  `cargo nextest run -p nako-addon-protocol --no-fail-fast`
- Contract consumers:
  `cargo check -p nako-addon-client -p nako-server -p nako-api --tests`
