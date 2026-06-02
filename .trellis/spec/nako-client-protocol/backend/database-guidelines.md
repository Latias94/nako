# Database Guidelines

`nako-client-protocol` has no database ownership. It describes public wire
payloads after server/API layers have mapped domain and persistence records.

## Required Patterns

- Represent IDs as public strings.
- Keep pagination as `PageInfo`.
- Keep current-user state without principal or account internals.
- Keep playlist, playback, and renderer state public and user-safe.
- Let server/API code decide database lookups, authorization, and mapping.

## Forbidden Patterns

- Do not import repository traits, SQL adapters, database pools, or migrations.
- Do not expose raw database IDs beyond public string IDs.
- Do not expose principal IDs, internal access rows, or private ownership
  records.
- Do not leak source locator, filesystem path, or transcode output path fields.

## Contract Rules

- `UserPlaybackStateDto` hides principal/user identity and exposes item/source
  playback state only.
- `UserPlaylistDto` exposes current-user playlist facts without collection or
  principal internals.
- `TranscodeSessionDto` exposes state and failure category, not output paths.

## Tests Required

- DTO serialization tests proving sensitive fields are absent.
- Route inventory tests for public paths.
- Server/API mapping tests should live outside this crate.
