# Error Handling

Core failure handling is transport-neutral and redaction-safe.

## Required Patterns

- Return `MissingAccessToken` before starting an auth probe when the token is
  empty after trimming.
- Return `HttpError` for non-2xx responses and include sanitized public error
  body when available.
- Return `UnsupportedApiVersion` when response header or health body reports a
  version other than `CLIENT_PROTOCOL_VERSION`.
- Return `InvalidResponse` for invalid health JSON or unknown probe response ID.
- Attach safe request previews to failures when available.

## Forbidden Patterns

- Do not panic on invalid JSON, unknown request IDs, or missing headers.
- Do not include raw access tokens in public errors or request previews.
- Do not require a transport error type in core failures.
- Do not treat a successful status with unsupported API version as success.

## Examples

- Health `200` with body version `v2` fails as unsupported API version.
- Auth probe `401` with message containing a token returns a redacted public
  error.
- Unknown probe `request_id` fails as invalid response.

## Review Checklist

- Is the failure kind specific?
- Is the token redacted in every returned preview and public error?
- Does the caller have enough information to retry or report?
