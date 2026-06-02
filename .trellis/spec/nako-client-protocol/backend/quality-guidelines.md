# Quality Guidelines

Public client protocol changes must preserve stable wire shape, route inventory,
and client safety.

## Required Patterns

- Add serde tests for each new DTO or public enum.
- Add route inventory tests when adding or changing a public path.
- Add sensitive-field absence tests for playback, streaming, renderer,
  transcode, user playback, and playlist DTOs.
- Preserve unknown additive string values with `Other(String)` where clients
  need forward compatibility.
- Keep Cargo dependencies limited to protocol needs.

## Forbidden Patterns

- Do not add server, API, database, transport, or core domain dependencies.
- Do not expose raw locators, bearer tokens, source paths, output paths,
  principal IDs, or private access records.
- Do not skip tests for public route and wire-string changes.
- Do not make a streaming route look like a JSON SDK method.

## Tests Required

- `public_route_inventory_is_protocol_owned_and_complete`.
- Public DTO serialization tests.
- Unknown wire string round-trip tests.
- Sensitive field absence tests.
- Error code conversion tests.

## Gate Selection

- Focused:
  `cargo nextest run -p nako-client-protocol --no-fail-fast`
- Consumers:
  `cargo nextest run -p nako-client-protocol -p nako-client-core -p nako-client --no-fail-fast`
