# Database Guidelines

`nako-addon-protocol` has no database ownership. It defines serializable
contracts and validation rules only.

## Required Patterns

- Represent accepted grants as caller-supplied scope slices.
- Use manifest declarations to validate declared requirements, not persisted
  authorization state.
- Keep Addon Token, grant storage, audit rows, and lifecycle state in server or
  database crates.
- Keep wire payloads serializable without repository dependencies.

## Forbidden Patterns

- Do not import SQL crates, repository traits, database pools, or migrations.
- Do not persist manifest validation results from this crate.
- Do not treat manifest `scopes` as accepted grants.
- Do not encode database IDs as protocol-only decisions when domain IDs belong
  to `nako-core`.

## Contract Rules

- `ensure_scope_grant`, `ensure_task_scope_grant`, and
  `ensure_event_subscription_scope_grant` validate requested grants against
  declarations.
- `validate_install_descriptor` verifies runtime and secret reference bindings,
  but does not install or register an addon.
- Protected writes are represented as side-effect payloads; Nako-owned APIs
  perform persistence and audit.

## Tests Required

- Manifest validation tests for scope declaration and grant checks.
- Install descriptor tests for secret references and runtime references.
- Side-effect payload wire-shape tests.
