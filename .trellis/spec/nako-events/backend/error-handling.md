# Error Handling

Webhook delivery should make subscription, retry, signing, transport, and
persistence failures distinguishable.

## Required Patterns

- Return `NakoError::InvalidInput` for invalid subscription configuration.
- Return `NakoError::Conflict` when max attempts are exhausted.
- Convert reqwest transport failures into provider-style errors.
- Persist failed attempts with redaction-safe failure details.
- Keep non-2xx transport responses visible as delivery failures.

## Forbidden Patterns

- Do not retry disabled subscriptions.
- Do not swallow persistence errors after a transport request.
- Do not expose webhook secret values in errors.
- Do not parse or serialize JSON with unchecked unwraps.

## Examples

- Disabled subscription: fail before signing or sending.
- Exhausted attempts: conflict, no extra transport call.
- Transport timeout: record failed attempt and return a provider error.

## Review Checklist

- Was an attempt persisted exactly once for this delivery path?
- Are secrets excluded from errors?
- Can the scheduler decide whether to retry?
