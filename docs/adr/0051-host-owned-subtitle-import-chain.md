# 0051: Host-Owned Subtitle Import Chain

## Status

Accepted.

## Context

Nako now has a read-only official subtitle provider addon and the Addon
Protocol already has `subtitle`, `subtitle_read`, and `subtitle_write`
vocabulary. The missing product chain is not only provider search. A complete
subtitle flow needs candidate discovery, user or policy selection, target
derivation, content validation, sidecar file write, library refresh, and
playback planning.

Letting a provider sidecar write subtitle files directly would give the addon
media-library filesystem authority and would bypass Nako's VFS, backup,
idempotency, audit, and path-redaction boundaries. The existing Library File
Write lane intentionally deferred subtitle writes until Nako owned a
first-party subtitle and track model.

## Decision

Nako owns subtitle import and library file writes. Subtitle provider addons may
search and fetch subtitle candidates, but they do not derive library paths or
write media-source sidecar files.

The shared Addon Protocol owns the first stable subtitle search wire types:
request, response, candidate, format, delivery, provider execution, and schema
constants. Official subtitle provider implementations must consume those
protocol types instead of carrying private duplicate schemas.

The host subtitle chain is split into explicit stages:

1. Search: Nako calls `AddonResource::Subtitle` providers and receives typed
   candidates.
2. Selection: Nako records a host-owned selected subtitle candidate reference.
3. Import plan: Nako derives the target media source, sidecar role, filename,
   language, format, overwrite policy, and backup policy without addon-provided
   filesystem paths.
4. Apply: Nako writes through Library File Write/VFS with atomic replace,
   backup, idempotency, and redacted reports.
5. Refresh: Nako refreshes library subtitle facts and exposes them to playback
   planning.

The first implementation slice may stop after the shared protocol contract and
official addon migration. Real sidecar writes, automatic download tasks, and
playback subtitle execution remain follow-on work.

## Consequences

- Subtitle providers remain small, testable, and read-only by default.
- Nako can support local, remote, read-only, and future cloud-backed libraries
  without giving addons unmediated storage access.
- The public subtitle wire shape stops drifting between Nako core and official
  addon crates.
- A later subtitle write implementation can reuse Library File Write rather
  than inventing an addon-specific path writer.

## Alternatives Considered

- Provider writes files directly: rejected because it leaks media-library
  storage authority and bypasses host audit and backup policy.
- Keep subtitle schemas private to each provider: rejected because Nako needs a
  stable host product flow and catalog drift prevention.
- Implement subtitle writes in the first slice: rejected because it needs target
  derivation, content validation, Library File Write semantics, and playback
  refresh as a separate bounded lane.

## Related Workstreams

- `docs/workstreams/subtitle-complete-chain/`
- `docs/workstreams/addon-library-file-write-policy/`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/adr/0035-addon-native-metadata-writeback.md`
