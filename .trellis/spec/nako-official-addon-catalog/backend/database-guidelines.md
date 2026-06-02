# Database Guidelines

`nako-official-addon-catalog` has no persistence. It supplies typed facts that
server/admin code can store or present elsewhere.

## Required Patterns

- Return `AddonManifest` and `AddonInstallDescriptor` values only.
- Let server/admin workflows decide registration, grants, token rotation, and
  installed-addon state.
- Use `AddonInstallGuide` through protocol helpers when tests need operator
  guidance output.
- Keep official addon facts deterministic and side-effect free.

## Forbidden Patterns

- Do not import repository traits, SQL adapters, database pools, or migrations.
- Do not write installed addon rows from this crate.
- Do not resolve or store secret values here.
- Do not treat catalog descriptors as accepted grants.

## Contract Rules

- Official manifests declare scopes; accepted grants are still server-owned.
- Secret reference fields describe expected references, not secret values.
- Binary and container descriptors are install guidance, not lifecycle
  automation.

## Tests Required

- Manifest validation tests for every official module.
- Descriptor validation tests for binary/container paths.
- Install guide tests for declared resources, tasks, events, and redaction.
