# Database Guidelines

`nako-events` uses repository traits for subscriptions and delivery attempts. It
does not own SQL schema in this crate.

## Required Patterns

- Load webhook subscriptions through core repository contracts.
- Persist each delivery attempt with enough status to audit success, failure,
  and retry eligibility.
- Enforce subscription enabled state before transport execution.
- Enforce max-attempt limits before building a signed request.
- Store safe failure details, not raw secrets or full payload dumps.

## Forbidden Patterns

- Do not import SQL adapters or connection pools.
- Do not deliver events without recording attempt state.
- Do not store webhook secrets in logs or failure strings.
- Do not treat transport success as persistence success.

## Retry Rules

- Use exponential retry delay starting from 30 seconds.
- Cap the exponent at the current implementation limit.
- Report exhausted attempts as a conflict rather than retrying forever.

## Tests Required

- Repository fake tests for disabled subscriptions and max attempts.
- Attempt persistence tests for success and failure paths.
- Retry-delay tests for attempt number behavior.
