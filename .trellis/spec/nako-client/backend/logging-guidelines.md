# Logging Guidelines

`nako-client` should return typed errors and request facts. Applications decide
how to log them.

## Required Patterns

- Prefer public-safe fields: method, URL without tokens, status, API version,
  error code, and request kind.
- Redact Authorization headers and bearer token strings.
- Use streaming builder request facts for diagnostics, not raw secret logs.
- Let callers attach trace IDs or app-specific logging context.

## Forbidden Patterns

- Do not log bearer tokens, login passwords, playback tickets, range URLs with
  credentials, or full response bodies by default.
- Do not log reqwest errors with sensitive URLs without caller redaction.
- Do not use logs as a substitute for `NakoClientError`.

## Useful Fields

- `client.method`
- `client.path`
- `client.status`
- `client.api_version`
- `client.error_code`
