# Admin Web Console Handoff

Status: Proposed
Last updated: 2026-05-17

## Current State

The workstream has been opened as a planning lane. It defines Taru's first web
surface as an admin console for media governance and operations, not the
flagship playback client.

AWC-020 is complete. `ADMIN_API_MATRIX.md` inventories current HTTP routes,
current DTO ownership, page coverage, safety behavior, and missing Admin API
surfaces.

The key artifact for v0.dev is:

- `docs/workstreams/admin-web-console/V0_CONTEXT.md`

## Next Recommended Task

Run AWC-030:

- decide whether the Admin API needs an ADR or a workstream design note;
- choose a route namespace/versioning direction;
- define Admin API DTO ownership and leakage rules;
- keep admin-only diagnostics out of the Public Client API.

## Constraints

- Do not choose a front-end framework until the user asks for implementation or
  accepts a stack.
- Do not place generated UI inside Rust server crates.
- Do not expose secrets, tokens, resolved provider credentials, webhook
  secrets, addon tokens, or unsafe local paths in UI contexts or mock data.
- Do not copy Jellyfin, Plex, or reference-project UI/source/assets.

## Open Questions

- Admin API namespace and versioning.
- First web app workspace path.
- Static mock prototype versus API-wired first slice.
- Editable settings versus read-only diagnostics in the first release.
