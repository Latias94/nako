# Directory Structure

`nako-client` currently keeps the SDK in `src/lib.rs`. Split only when query
types, JSON methods, streaming builders, transport, and error mapping become too
large to review in one file.

## Current Layout

- Protocol DTO re-exports.
- `NakoClient` constructor, bearer token setter, async JSON methods, and
  streaming request builders.
- Query parameter structs such as `PageQuery`, `LibraryItemsQuery`,
  `SearchQuery`, `PlaybackCapabilitiesQuery`, `ImageVariantQuery`, and
  `RemuxPlaybackQuery`.
- `ClientRequest`, `ClientResponse`, `ClientTransport`, and `ReqwestTransport`.
- Core request/response conversion helpers.
- `NakoClientError`.
- Mock transport tests.

## Module Split Rules

- Keep public DTO re-exports centralized.
- Move transport implementations into a private transport module before adding a
  second transport.
- Move query structs by route family only if SDK methods are also grouped.
- Keep error mapping close to core response conversion.

## Naming Rules

- Use `NakoClient::*` for public SDK operations.
- Use `*_request` suffix for streaming builders that return `ClientRequest`.
- Use `Query` suffix for borrowed query structs.
- Use `ClientTransport` for mockable send behavior.

## Anti-Patterns

- Do not add CLI argument parsing here.
- Do not add UniFFI records here.
- Do not hide route paths in transport code.
