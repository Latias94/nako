# Database Guidelines

Automation persistence is accessed through core repository and job contracts.
This crate does not own SQL schema.

## Required Patterns

- Enqueue automation work as durable `JobKind::Automation`.
- Use `JobPriority::Normal` unless a real priority policy exists.
- Persist provider artifacts through repository contracts.
- Start, complete, or fail jobs through the durable job API.
- Keep provider secrets out of stored error details.

## Forbidden Patterns

- Do not import SQL adapters, database pools, or migrations.
- Do not run provider work without a corresponding job record.
- Do not mutate canonical metadata from automation artifact creation.
- Do not persist raw provider secret material.

## Job Rules

- Validate provider enabled state and capability before enqueue.
- Use resource class `automation.external_api`.
- Respect max attempts and timeout configuration.
- Mark failed jobs with redaction-safe `safe_error` values.

## Tests Required

- Fake repository tests for enqueue validation.
- Job lifecycle tests for start, success, failure, and cancellation.
- Artifact persistence tests for provider outcomes.
