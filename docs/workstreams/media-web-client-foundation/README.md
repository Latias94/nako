# Media Web Client Foundation

Status: Active
Last updated: 2026-05-26

This workstream owns the first browser-based Media Web surface for local media
browsing and playback through the Public Client API.

It is split from `client-surface-and-access-product-architecture` after the
identity/access contract lane made Users, Roles, Library Access, bootstrap
administrator semantics, and Public Client API effective-access enforcement
durable enough for a real client surface.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`

The first product slice is deliberately local-media-first:

- Admin and Media coexist in one frontend project with separate route
  namespaces and module boundaries;
- connect/login using the current accepted access model;
- Libraries and Media Library detail;
- search and Media Item detail;
- Source/Version Picker;
- browser playback through Public Client API playback decisions;
- User Playback State and Continue Watching when available;
- no Admin API state, no public registration, no recommendations, and no
  streaming-storefront discovery.
