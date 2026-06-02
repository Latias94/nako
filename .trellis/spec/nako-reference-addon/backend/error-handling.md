# Error Handling

The reference addon intentionally keeps error behavior minimal. It should remain
a deterministic fixture, not a production resilience layer.

## Required Patterns

- Validate fixture manifest shape in tests.
- Keep route handlers total for simple JSON requests.
- Use default fallback title `Unknown Title` when metadata request payload lacks
  a title.
- Let protocol/client/server tests assert error handling around this fixture.

## Forbidden Patterns

- Do not add provider-style retries or transport errors here.
- Do not panic on missing optional metadata request fields.
- Do not return production-only error shapes that the protocol crate does not
  define.
- Do not introduce auth failure paths in the no-auth fixture.

## Examples

- A metadata request with title `The Matrix` returns that title in payload.
- A metadata request without title returns `Unknown Title`.
- Health response returns `AddonHealthStatus::Ok`.

## Review Checklist

- Is the fixture still deterministic?
- Are protocol facts still echoed correctly?
- Are errors still tested in the client/server layer instead?
