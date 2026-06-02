# Error Handling

`NakoClientError` is the SDK error vocabulary. Preserve clear separation between
local construction, transport, API, version, encode, and decode failures.

## Required Patterns

- Use `InvalidBaseUrl` when client construction cannot parse the base URL.
- Use `InvalidPath` and `InvalidHeader` for request construction failures.
- Use `Transport` only for reqwest send/body read failures.
- Use `Api { status, body }` for non-2xx API responses after public error
  parsing.
- Use `UnsupportedApiVersion` when `x-nako-api-version` does not match.
- Use `Encode` and `Decode` for JSON body issues.
- Use `MissingAccessToken` for authenticated calls without token.

## Forbidden Patterns

- Do not expose bearer token values in error strings.
- Do not collapse API errors into transport errors.
- Do not skip API version checks on successful responses.
- Do not parse JSON with unchecked unwraps outside tests.

## Examples

- API `403` with public `ErrorResponse` becomes `NakoClientError::Api`.
- Response header `x-nako-api-version: v2` becomes unsupported API version.
- Invalid response JSON becomes `Decode`.

## Review Checklist

- Is this a construction, transport, API, version, encode, or decode failure?
- Is token material absent from the error?
- Does the mock transport test cover the path?
