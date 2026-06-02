# Logging Guidelines

`nako-client-uniffi` should not log. It returns request and failure records to
foreign-language callers.

## Required Patterns

- Use `CoreSafeRequestPreview` for diagnostics.
- Let mobile/foreign clients own logging policy.
- Preserve redacted headers and sanitized URLs from core.

## Forbidden Patterns

- Do not log access tokens, bearer headers, playback tickets, raw response
  bodies, or foreign client secrets.
- Do not print binding diagnostics during request construction.
- Do not use logs as a substitute for returned failure records.

## Useful Fields

- `client_uniffi.request_id`
- `client_uniffi.method`
- `client_uniffi.safe_url`
- `client_uniffi.failure_kind`
