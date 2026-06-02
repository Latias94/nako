# Logging Guidelines

`nako-client-protocol` should not normally log. Public DTOs and route inventory
are validated by tests.

## Required Patterns

- Prefer tests and explicit DTO fields over logs.
- If diagnostics are added, log only public-safe facts such as route path,
  route kind, method, API version, DTO family, and error code.
- Keep public error messages redaction-safe.

## Forbidden Patterns

- Do not log login passwords, session tokens, playback tickets, source locators,
  renderer transport secrets, or server filesystem paths.
- Do not log entire response DTOs in protocol tests.
- Do not use logs to detect public route drift.

## Useful Fields

- `client_protocol.version`
- `client_protocol.route`
- `client_protocol.method`
- `client_protocol.route_kind`
- `client_protocol.error_code`
