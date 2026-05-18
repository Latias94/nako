# Admin Web Console Handoff

Status: Active
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

AWC-035 is complete for M52. `GET /admin/v1/overview` is the first code-backed
Admin API v1 seam. The route is read-only and composes existing safe storage,
metadata-provider, runtime, and startup diagnostics through admin-owned DTOs in
`taru-api::admin`.

The key context artifact for v0.dev is:

- `docs/workstreams/admin-web-console/V0_CONTEXT.md`

AWC-040 and AWC-050 are complete for M53. `V0_CONTEXT.md` now distinguishes
the live `GET /admin/v1/overview` seam from remaining mock or planned Admin API
surfaces, and this handoff captures the first concise v0.dev prompt.

## v0.dev Prompt

```text
Create a polished first prototype for Taru, a self-hosted media server admin
console. Taru should feel like a quiet private media cellar: refined,
preservation-focused, privacy-first, and operationally clear, not like a
streaming storefront or SaaS landing page.

Build an app shell with left navigation for Overview, Media Libraries, Catalog,
Metadata, Playback & Transcode, Storage, Automation, Addons, Network, and
Settings. Focus the prototype pages on Overview, Media Libraries, Library
Detail, Metadata Providers, Jobs/Tasks, Playback & Transcode, and Settings.

Use dense but calm admin UI patterns: tables, filters, tabs, status badges,
detail drawers, safe error states, and concise actions. The Overview page is
partially live via GET /admin/v1/overview for server/API version, storage,
metadata-provider, runtime, and startup summaries. Use realistic mock data for
job lists, session lists, event histories, hardware dashboards, settings,
catalog repair, Addons, Automation, and Network until follow-up Admin API
routes exist.

Use Taru domain language: Media Library, Media Source, Media Item, Canonical
Metadata, Provider Mapping, Local Inference, NFO, Playback Source Selection,
Addon Sidecar, and Automation Provider. Do not show plaintext secrets, tokens,
resolved provider keys, webhook secrets, addon tokens, unsafe local paths, or
raw provider bodies. Keep hosted addon pages clearly external. Do not choose or
describe a front-end framework; produce the UI prototype only.
```

## Next Recommended Task

AWC-060 is the next implementation task, but it is intentionally blocked until
the user accepts a front-end stack and workspace location. Do not scaffold UI
code before that decision.

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
