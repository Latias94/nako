# Logging Guidelines

The reference addon should not need routine logging. Tests should inspect
returned protocol payloads directly.

## Required Patterns

- Prefer deterministic response payload assertions over logs.
- If route diagnostics are added, log only fixture-safe fields such as request
  ID, resource kind, route name, and addon ID.
- Keep request payloads out of logs unless a test explicitly verifies safe
  fixture content.

## Forbidden Patterns

- Do not log Addon Tokens or auth headers; this fixture should not require them.
- Do not log full request bodies from tests by default.
- Do not log local filesystem paths or generated side-effect payload internals.
- Do not rely on logs to prove route behavior.

## Useful Fields

- `reference_addon.id`
- `reference_addon.route`
- `reference_addon.request_id`
- `reference_addon.resource`
