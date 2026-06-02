# Logging Guidelines

`nako-addon-client` should expose safe error codes and outcomes to callers
instead of logging raw HTTP details internally.

## Required Patterns

- Prefer safe fields: addon ID, resource, task ID, subscription ID, request ID,
  status, attempt count, and `safe_code`.
- Let server/control-plane code attach job IDs, trace IDs, and persistence
  context.
- Redact URLs, tokens, shared secrets, query parameters, request bodies, and raw
  addon responses by default.
- Use outcome structs for caller-visible diagnostics.

## Forbidden Patterns

- Do not log `authorization`, `x-nako-addon-secret`, Addon Token values, or full
  request bodies.
- Do not log reqwest error URLs.
- Do not log full external acquisition materialized links or subtitle bodies.
- Do not use logs as a substitute for returned attempt counts.

## Useful Fields

- `addon.id`
- `addon.resource`
- `addon.task_id`
- `addon.subscription_id`
- `addon.request_id`
- `addon.attempts`
- `addon.error_code`
