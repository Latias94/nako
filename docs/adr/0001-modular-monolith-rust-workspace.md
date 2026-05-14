# 0001: Use a Modular Monolith Rust Workspace

## Status

Proposed

## Context

Taru needs media scanning, metadata resolution, NFO import/export, search,
playback decisions, transcoding, webhooks, automation, and eventually remote
storage and addons. These concerns should be isolated early, but operating them
as separate services would add deployment complexity before the domain model is
stable.

## Decision

Use a Rust workspace with multiple internal crates and a single server binary
for the MVP. Keep crate dependency direction explicit:

- API depends on application/domain services, not database internals.
- Domain models and traits live in `taru-core`.
- Infrastructure crates implement traits from core or domain crates.
- Cross-cutting work such as events, storage, search, and transcode gets its
  own crate boundary.

## Consequences

- The first deployment remains simple.
- Internal APIs can evolve faster than network protocols.
- Future service extraction remains possible when a boundary is proven by load,
  ownership, or deployment needs.
- The workspace may contain more crates than a minimal project, so dependency
  direction needs regular review.

## Alternatives Considered

- Single crate: easier initially, but likely to entangle scanning, playback,
  storage, and metadata.
- Microservices: clearer deployment isolation, but premature for the MVP and
  harder for self-hosted users.

## Related Workstreams

- `docs/workstreams/server-foundation/`
