# Admin Web V2 Media Browsing And Item Detail Governance

Status: Active
Last updated: 2026-05-25

This workstream builds the next Admin Web V2 follow-on after library management:
governance-oriented media browsing and item detail. It gives operators a safe
way to inspect Media Items, Media Sources, Canonical Metadata, artwork/NFO
readiness, and support evidence without turning Admin Web into the playback
client.

Authoritative docs:

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [ROUTE_API_READINESS.md](ROUTE_API_READINESS.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [WORKSTREAM.json](WORKSTREAM.json)
- [HANDOFF.md](HANDOFF.md)

## Why This Lane Exists

The closed library-management lane delivered `/libraries/:libraryId`, Metadata
Profile editing, scan/NFO commands, Source inventory bridge summaries, and the
first Admin Web i18n boundary. Operators still cannot use V2 to inspect a Media
Item in context: sources, metadata authority, artwork state, NFO/provider
evidence, catalog relationships, or playback support links.

This lane adds that inspection layer as a route-owned governance workflow. It
does not add watch-first browsing, playback controls, settings mutation, user
state, or repair/apply actions.
