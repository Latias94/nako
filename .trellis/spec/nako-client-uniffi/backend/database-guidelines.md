# Database Guidelines

`nako-client-uniffi` has no persistence. Foreign clients receive request facts
and decide how to execute them.

## Required Patterns

- Treat IDs and tokens as opaque input strings.
- Return `CoreHttpRequest` records with safe previews.
- Return `CoreRuntimeFailure` records for interpreted failures.
- Let the server and foreign client runtime own persistence and transport.

## Forbidden Patterns

- Do not import repository traits, SQL adapters, database pools, or migrations.
- Do not persist access tokens or connection state.
- Do not execute requests from bindings.
- Do not expose database concepts through binding records.

## Contract Rules

- Connection probe starts with an unauthenticated health request.
- Auth probe and authenticated route builders include bearer headers in the raw
  request and redacted bearer headers in `safe_preview`.
- Streaming builders follow core behavior and do not add database or auth policy.

## Tests Required

- Binding request builder tests should compare request IDs, URLs, methods, and
  safe previews against expected core output.
