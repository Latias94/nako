# Logging Guidelines

Webhook logs must be useful for delivery diagnostics while excluding secrets and
raw payload content.

## Required Patterns

- Prefer structured fields for event type, subscription ID, attempt number,
  status, and retry delay.
- Log payload size or event ID instead of full payloads.
- Redact webhook secrets, signatures, and authorization headers.
- Keep transport error details safe before persistence or logging.

## Forbidden Patterns

- Do not log full signed webhook payloads.
- Do not log `x-nako-signature` values.
- Do not log webhook target credentials.
- Do not use logs as the only evidence of delivery attempts.

## Useful Fields

- `events.event_type`
- `events.subscription_id`
- `events.attempt_number`
- `events.delivery_status`
- `events.retry_after`
