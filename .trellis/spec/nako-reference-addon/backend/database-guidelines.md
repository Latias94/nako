# Database Guidelines

`nako-reference-addon` has no database ownership. It is a sidecar fixture that
returns protocol payloads.

## Required Patterns

- Keep route handlers stateless.
- Return deterministic payloads based on request JSON only.
- Use protocol payload types for demo protected writes.
- Let server tests own persistence assertions when they call this fixture.

## Forbidden Patterns

- Do not import repository traits, SQL adapters, database pools, or migrations.
- Do not persist addon responses.
- Do not write NFO, artwork, subtitle, or metadata state from this fixture.
- Do not simulate accepted grants by mutating global state.

## Contract Rules

- `/metadata` returns an `AddonResourceResponse` with the request's addon ID,
  resource, and request ID.
- `/health` returns manifest facts for addon version and resource count.
- Demo NFO payload uses `AddonLibraryFileRole::Nfo` and
  `AddonLibraryFileWritePolicy::CreateMissing`.

## Tests Required

- Manifest validation tests.
- Protected write payload serialization tests.
- Server/database persistence behavior should be tested outside this crate.
