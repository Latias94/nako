# Logging Guidelines

`nako-client-core` should return safe previews and failures instead of logging.

## Required Patterns

- Use `CoreSafeRequestPreview` for diagnostics.
- Redact Authorization headers as `Bearer <redacted>`.
- Sanitize known secrets in URLs and public error messages.
- Let transport adapters or applications own logging.

## Forbidden Patterns

- Do not log raw access tokens, bearer strings, full request bodies, or playback
  tickets.
- Do not expose raw request headers in failures without redaction.
- Do not use logs as the only source of connection probe state.

## Useful Fields

- `client_core.request_id`
- `client_core.method`
- `client_core.safe_url`
- `client_core.failure_kind`
- `client_core.observed_api_version`
