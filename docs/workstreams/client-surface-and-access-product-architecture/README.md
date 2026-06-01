# Client Surface And Access Product Architecture

Status: Closed
Last updated: 2026-06-01

This planning lane defines how Nako should grow from Single-Admin Mode and
Admin Web V2 into a coherent set of user-facing and operator-facing client
surfaces.

It covers:

- the product boundary between Admin Web and Media Web;
- the staged account, role, and Library Access model;
- admin-to-media and media-to-admin context switching;
- desktop client strategy with Tauri plus a native playback core;
- mobile/native boundaries that remain separate from server administration;
- first implementation slices that keep local media playback ahead of
  recommendation or streaming-storefront features.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CONTEXT.jsonl`
- `HANDOFF.md`
- `CLOSEOUT.md`

The lane is product-architecture first and is now closed. Identity/access is now owned by the
completed `identity-and-library-access-contract` lane, and browser Media Web is
owned by the closed `media-web-client-foundation` foundation lane and current
`web/` follow-ons. Management Context Links are now split to
`admin-media-management-context-links`. Desktop playback strategy is explicitly
deferred to a future focused Tauri/native playback spike rather than keeping
this broad planning lane active.
