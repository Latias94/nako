# Quality Guidelines

Official addon catalog changes must prevent fact drift and keep install guidance
redaction-safe.

## Required Patterns

- Test each module's default manifest against its constants.
- Test container descriptors through `addon_install_guide`.
- Test resource order when callers rely on deterministic declared resource
  lists.
- Test configuration schema defaults for provider toggles, limits, timeouts, and
  runner profiles.
- Keep official addon versions aligned with runtime image and install command
  facts.

## Forbidden Patterns

- Do not update `ADDON_VERSION` without matching runtime image/install facts.
- Do not put real external API secrets in config schema defaults.
- Do not add official addon scopes without manifest and descriptor tests.
- Do not make tests depend on running sidecar services.

## Tests Required

- One default manifest test per official addon module.
- Binary and container install descriptor tests.
- Schema default tests for provider and runner configuration.
- Secret reference redaction tests when a module declares secret fields.

## Gate Selection

- Focused:
  `cargo nextest run -p nako-official-addon-catalog --no-fail-fast`
- Official addon protocol:
  `cargo nextest run -p nako-addon-protocol -p nako-official-addon-catalog --no-fail-fast`
