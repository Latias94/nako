# Quality Guidelines

Reference addon changes should preserve fixture simplicity and protocol
coverage.

## Required Patterns

- Keep manifest validation tests passing.
- Assert entry point, hosted page, and configuration schema IDs.
- Assert demo protected write payload serialization.
- Keep metadata suggestion artifact shape stable unless protocol tests are
  updated.
- Keep fixture behavior independent from external services.

## Forbidden Patterns

- Do not add live network or provider dependencies.
- Do not add real filesystem, VFS, database, or NFO writes.
- Do not broaden fixture behavior without a test showing which protocol contract
  it exercises.
- Do not depend on official addon catalog constants.

## Tests Required

- `reference_manifest_is_valid`.
- Protected write payload contract tests.
- Route-level tests if route behavior changes.
- Cross-crate client/server tests if this fixture is used for integration.

## Gate Selection

- Focused:
  `cargo nextest run -p nako-reference-addon --no-fail-fast`
- Fixture consumers:
  `cargo check -p nako-reference-addon -p nako-addon-client -p nako-server --tests`
