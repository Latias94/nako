# Client Surface And Access Product Architecture

Status: Draft
Last updated: 2026-05-26

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
- `HANDOFF.md`

The lane is product-architecture first. It does not implement account storage,
new Public Client API routes, Admin Web controls, Media Web, Tauri packaging,
or native player integration until follow-on tasks accept narrower contracts.
