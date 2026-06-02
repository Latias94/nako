# Logging Guidelines

Automation diagnostics must show job progress without exposing external
provider secrets or raw responses.

## Required Patterns

- Prefer structured fields for provider ID, job ID, capability, attempt number,
  status, and artifact ID.
- Use redaction-safe error text for job failure logs.
- Log cancellation and timeout as job lifecycle events.
- Keep provider request and response bodies out of normal logs.

## Forbidden Patterns

- Do not log API tokens, provider secrets, or authorization headers.
- Do not log raw external API responses by default.
- Do not replace durable job state with logs.
- Do not emit high-cardinality per-record logs for bulk provider output.

## Useful Fields

- `automation.provider_id`
- `automation.job_id`
- `automation.capability`
- `automation.attempt_number`
- `automation.artifact_id`
