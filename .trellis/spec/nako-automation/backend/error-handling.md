# Error Handling

Automation errors should preserve job lifecycle state and protect provider
secrets.

## Required Patterns

- Fail enqueue when provider is disabled or lacks the requested capability.
- Respect provider timeout and cancellation signals.
- Record provider failures in durable job state with safe error text.
- Return provider execution failures after recording the failed job state.
- Reject direct canonical metadata acceptance with `NakoError::InvalidInput`.

## Forbidden Patterns

- Do not unwrap provider results or cancellation checks.
- Do not expose provider secrets in returned errors.
- Do not mark failed provider execution as a successful artifact.
- Do not ignore durable job persistence errors.

## Examples

- Disabled provider: fail before enqueueing a job.
- Provider timeout: fail job and return a provider-style error.
- Outcome requesting canonical acceptance: fail as invalid input during M5.3.

## Review Checklist

- Is the job state consistent with the returned result?
- Are secrets excluded from errors and artifacts?
- Can a scheduler retry or inspect the failure?
