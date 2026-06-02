# Error Handling

Public client errors use `ErrorResponse` plus `ClientErrorCode` strings. This
crate defines the public vocabulary, not server-side error generation.

## Required Patterns

- Add new client-visible error categories to `ClientErrorCode::ALL`.
- Keep `ClientErrorCode::as_str` aligned with serde snake-case values.
- Use `ErrorResponse::new` when constructing public errors in tests.
- Keep client error messages public-safe.
- Keep storage, provider, FFmpeg, staging, auth, and database categories
  explicit when they are exposed to public clients.

## Forbidden Patterns

- Do not expose raw database or storage errors in public DTOs.
- Do not encode private exception type names as client error codes.
- Do not remove an error code without a migration plan.
- Do not treat unknown playback wire strings as fatal when the enum is additive.

## Examples

- `"not_found"` maps to `ClientErrorCode::NotFound`.
- Unknown playback mode strings round-trip through `Other(String)`.
- A public error body contains `code` and `message`, not stack traces.

## Review Checklist

- Is the error code stable and public-safe?
- Does `ClientErrorCode::from_code` still find it?
- Are unknown additive strings preserved where needed?
