# Database Guidelines

`nako-client` has no persistence. It calls the Public Client API and decodes
public DTOs.

## Required Patterns

- Treat all IDs as opaque public strings.
- Encode IDs before putting them into paths.
- Let the server enforce access, pagination limits, playlist versions, playback
  policy, and persistence.
- Return public DTOs from `nako-client-protocol`.
- Keep transport responses in memory only.

## Forbidden Patterns

- Do not import repository traits, SQL adapters, database pools, or migrations.
- Do not cache sessions, playlists, playback state, or media metadata here.
- Do not infer server persistence behavior from public DTOs.
- Do not persist bearer tokens in the SDK.

## Contract Rules

- `with_bearer_token` stores token in the client instance for request headers.
- `health` and `login` are unauthenticated.
- Authenticated JSON methods require an Authorization header.
- Streaming request builders return `ClientRequest` so applications can handle
  byte/range transport themselves.

## Tests Required

- Mock transport tests for each new JSON method.
- Streaming builder tests for path/query/header behavior.
- Persistence behavior remains server/API test responsibility.
