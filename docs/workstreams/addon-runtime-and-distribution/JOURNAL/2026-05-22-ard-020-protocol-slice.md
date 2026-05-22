# ARD-020 Protocol Slice

Date: 2026-05-22

## Work Completed

- Added `AddonInstallDescriptor`, `AddonRuntimeRequirement`,
  `AddonRuntimeReference`, Secret Reference binding, and `AddonInstallGuide`
  DTOs to `taru-addon-protocol`.
- Added validation for:
  - missing or multiple runtime references;
  - local paths, URLs, env flags, and credential-bearing runtime references;
  - unknown or duplicate Secret Reference bindings;
  - likely raw secret values in bindings while allowing explicit reference
    forms such as `env:TARU_METADATA_ADDON_TOKEN`.
- Added install-guide generation that summarizes manifest, runtime, scope,
  task, event, and Secret Reference facts without resolving or storing secret
  values.

## Validation

- `cargo nextest run -p taru-addon-protocol install_descriptor --no-fail-fast`
- `cargo nextest run -p taru-addon-protocol --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`
- `git diff --name-only -- crates/taru-client-protocol`

## Remaining Work

ARD-020 remains open. Next step is an Admin DTO/server install-guide preview
boundary with redaction-focused HTTP tests.
