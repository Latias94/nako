# Error Handling

Client errors must be specific enough for schedulers and callers while staying
redaction-safe.

## Required Patterns

- Use `AddonClientError::Protocol` for manifest and envelope validation errors.
- Use `InvalidRequest` for specialized helper request-schema failures.
- Use `InvalidResponse` for response payload/schema failures.
- Use `UnsafeRequestBody` when request bodies contain token material.
- Use `HttpStatus { status, retryable }` for non-2xx HTTP responses.
- Use `Http { message }` for transport failures after redaction.

## Retry Matrix

- Retryable: transport errors, HTTP 408, HTTP 429, and HTTP 5xx.
- Not retryable: protocol errors, invalid request, invalid response, unsafe
  body, and non-retryable HTTP statuses.
- Setup failures return attempts `0`; dispatched failures return the last
  attempted count.

## Forbidden Patterns

- Do not include bearer tokens, shared secrets, query tokens, URLs, or local
  paths in error text.
- Do not retry invalid manifests or schema mismatches.
- Do not parse response JSON with unchecked unwraps.
- Do not drop attempt counts.

## Review Checklist

- Is the failure retryable or terminal?
- Is the attempt count correct?
- Does `safe_code()` classify the error for callers?
- Is sensitive material absent from the error string?
