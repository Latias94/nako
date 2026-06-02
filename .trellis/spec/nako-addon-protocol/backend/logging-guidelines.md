# Logging Guidelines

`nako-addon-protocol` should usually return validation errors instead of
logging. If diagnostics are added, they must be redaction-safe for addon author
tooling.

## Required Patterns

- Prefer structured validation facts: manifest ID, resource kind, task ID,
  event subscription ID, protocol version, and route kind.
- Keep raw payloads and external URLs out of logs.
- Use custom `Debug` implementations for sensitive payloads.
- Emit install-guide facts without plaintext secret values.

## Forbidden Patterns

- Do not log Addon Tokens, shared secrets, passwords, magnet links, renderer
  tickets, local filesystem paths, or full side-effect payloads.
- Do not log Addon Hosted Page URLs as trusted admin URLs.
- Do not log full manifests if they can include secret references or private
  deployment details.

## Useful Fields

- `addon.manifest_id`
- `addon.protocol_version`
- `addon.resource`
- `addon.task_id`
- `addon.event_subscription_id`
- `addon.route_kind`
