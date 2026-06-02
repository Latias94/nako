# Logging Guidelines

`nako-official-addon-catalog` should not normally log. It provides deterministic
facts for callers and tests.

## Required Patterns

- Prefer returned manifest/descriptor values over logs.
- If diagnostics are added, log addon ID, version, runtime kind, resource count,
  task count, event subscription count, and hosted page count.
- Keep install notes and secret reference names redaction-safe.
- Let server/admin code own operator-facing audit logs.

## Forbidden Patterns

- Do not log secret reference values.
- Do not log local runtime paths as accepted facts.
- Do not log full generated JSON schema bodies in normal operation.
- Do not use logs to detect catalog drift.

## Useful Fields

- `official_addon.id`
- `official_addon.version`
- `official_addon.runtime_kind`
- `official_addon.resource_count`
- `official_addon.task_count`
- `official_addon.event_subscription_count`
