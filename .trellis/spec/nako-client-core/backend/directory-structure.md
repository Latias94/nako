# Directory Structure

`nako-client-core` is organized by transport-neutral client responsibility.

## Current Layout

- `lib.rs`: re-exports the core API.
- `ids.rs`: stable request IDs.
- `encoding.rs`: percent encoding and base URL joining.
- `redaction.rs`: token and bearer redaction.
- `request.rs`: request specs, headers, query params, safe previews.
- `response.rs`: HTTP/API version failure interpretation.
- `connection.rs`: health and auth probe state machine.
- `browse.rs`: library, item, people, tag, genre, and search builders.
- `artwork.rs`: image request builder.
- `playback.rs`: source probe, playback decision, streaming, HLS, and session
  builders.
- `user_playback.rs`: current-user playback state builders.

## Module Rules

- Add new route builders to the module that matches the public API family.
- Add reusable request IDs to `ids.rs`.
- Keep redaction helpers private unless foreign-language clients need the exact
  behavior.
- Keep response interpretation generic and transport-neutral.

## Naming Rules

- Use `Core*Input` for builder inputs.
- Use `build_*_request` for request builders.
- Use `Core*Target` for playback target bundles that include optional preflight.
- Use `CoreRuntimeFailure*` for interpreted failures.

## Anti-Patterns

- Do not add HTTP client adapters here.
- Do not duplicate public DTOs from `nako-client-protocol`.
- Do not create server-specific route modules.
