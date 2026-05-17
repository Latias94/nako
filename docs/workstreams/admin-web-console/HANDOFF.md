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

AWC-030 is complete. ADR 0027 accepts `/admin/v1/*` as the Admin API boundary,
keeps admin DTOs in `taru-api`, keeps `taru-client-protocol` public-client-only,
and defines redaction/leakage rules for future Admin API slices.

The key artifact for v0.dev is:

- `docs/workstreams/admin-web-console/V0_CONTEXT.md`

## Next Recommended Task

Run AWC-040:

- refine `V0_CONTEXT.md` with ADR 0027's `/admin/v1/*` boundary;
- make clear which first prototype pages are mock-only versus backed by current
  routes;
- keep generated UI context framework-neutral and admin-focused.

## Constraints

- Do not choose a front-end framework until the user asks for implementation or
  accepts a stack.
- Do not place generated UI inside Rust server crates.
- Do not expose secrets, tokens, resolved provider credentials, webhook
  secrets, addon tokens, or unsafe local paths in UI contexts or mock data.
- Do not copy Jellyfin, Plex, or reference-project UI/source/assets.

## Open Questions

- First web app workspace path.
- Static mock prototype versus API-wired first slice.
- Editable settings versus read-only diagnostics in the first release.
