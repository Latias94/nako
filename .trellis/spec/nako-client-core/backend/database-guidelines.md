# Database Guidelines

`nako-client-core` has no persistence. It only builds requests and interprets
responses supplied by a caller.

## Required Patterns

- Treat IDs as opaque public path segments.
- Use percent encoding before putting IDs into paths.
- Use query parameters for page, facets, playback capabilities, and output
  container choices.
- Let the server enforce authorization, pagination limits, and persistence.
- Return request facts rather than mutating client state.

## Forbidden Patterns

- Do not import repository traits, SQL adapters, database pools, or migrations.
- Do not validate IDs by querying storage.
- Do not persist access tokens or connection probe state.
- Do not assume server database shape from public DTOs.

## Contract Rules

- Connection probe first calls unauthenticated `/health`, then authenticated
  `/libraries?limit=1&offset=0`.
- Streaming builders for direct/remux/HLS do not attach bearer auth by default.
- User playback write builders use `PUT` and `Content-Type: application/json`.

## Tests Required

- Request-builder tests should assert exact URLs and methods.
- Persistence and auth policy tests belong in server/API crates.
