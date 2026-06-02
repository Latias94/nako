# Quality Guidelines

Addon client changes must preserve mockability, schema validation, retry
behavior, and redaction.

## Required Patterns

- Use fake `AddonTransport` tests for request headers, body, URL, attempts, and
  responses.
- Assert `authorization`, `x-nako-addon-secret`, `x-nako-attempt`,
  `x-nako-request-id`, protocol, addon ID, and resource headers where relevant.
- Test specialized schema mismatch paths before calling the transport.
- Test retry behavior with both transport errors and retryable HTTP statuses.
- Test reqwest transport errors do not expose URL or query token material.

## Forbidden Patterns

- Do not add real network tests for normal client behavior.
- Do not make retry behavior nondeterministic.
- Do not bypass mock transport in new call helpers.
- Do not hide unsafe-body checks behind debug-only assertions.

## Tests Required

- Resource call success/failure tests.
- Resource-search, link-check, subtitle, task, event, and health helper tests.
- `NakoRuntimeClient` side-effect and generated-artifact call tests.
- Retry and non-retry tests.
- Redaction tests for request body and transport errors.

## Gate Selection

- Focused:
  `cargo nextest run -p nako-addon-client --no-fail-fast`
- Protocol:
  `cargo nextest run -p nako-addon-protocol -p nako-addon-client --no-fail-fast`
