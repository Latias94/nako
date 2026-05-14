# 0008: Treat NFO as a Local Metadata Boundary

## Status

Proposed

## Context

NFO files are common in self-hosted media libraries. They are useful for local
metadata control, migration from existing tools, and reproducible metadata next
to media files. Taru also needs to support future provider refreshes without
destroying local NFO decisions.

## Decision

Treat NFO as a local metadata source. Import/export is implemented through
`taru-nfo` behind a codec boundary. The first codec is a minimal movie NFO codec
covering core fields:

- title
- original title
- sort title
- overview
- release date
- runtime
- tagline
- genres

NFO import should produce canonical metadata plus local authority information.
Field-lock policy decides whether imported fields are protected from provider
refresh.

Soft-link and hard-link management is not part of the NFO codec. Link behavior
belongs to VFS/storage policy because link capability depends on backend type.

## Consequences

- NFO support can improve incrementally without coupling provider metadata to
  XML parsing.
- Local metadata workflows remain first-class.
- Remote storage backends can opt out of link-specific behavior.
- Full compatibility with other media servers remains future work.

## Alternatives Considered

- Parse every known NFO variant immediately: too broad for the foundation.
- Store NFO XML as canonical metadata: preserves data but makes API and merge
  policy harder.
- Treat NFO as just another remote provider: ignores local authority and file
  ownership semantics.

## Related Workstreams

- `docs/workstreams/server-foundation/`
