# Admin Web V2 Library Management And Localization

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

Admin Web V2 has a clean route-first shell and a read-only `/libraries` page,
but that page cannot yet answer the operator questions raised by a Jellyfin or
Plex style web console:

- What does this Media Library contain?
- Which sources are visible and when were they scanned?
- Which metadata language, provider order, NFO, and Addon scrape rules apply?
- Can I trigger scan, NFO import/export, or inspect related jobs safely?
- Can the console itself be localized without scattering more hard-coded copy?

The previous `/libraries` lane intentionally closed as read-only and split
metadata-profile editing, scan/NFO actions, public inventory, and richer source
counts into follow-ons. This lane owns those follow-ons as a bounded first
management workflow.

## Relevant Authority

- `CONTEXT.md`
- `PRODUCT.md`
- `DESIGN.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/workstreams/admin-web-console/`
- `docs/workstreams/admin-web-v2-product-architecture/`
- `docs/workstreams/admin-web-v2-media-libraries-route/`
- `docs/workstreams/admin-library-metadata-profile-configuration/`
- `docs/workstreams/metadata-profile-configuration-authority/`
- `docs/workstreams/multi-library-hardening/`
- `docs/api/HTTP_API.md`

## Problem

The current Admin Web V2 route set is good at showing redaction-safe
diagnostics, but not yet good at managing media libraries:

- `/libraries` lists configured libraries but has no detail route.
- `GET/PUT /admin/v1/libraries/{library_id}/metadata-profile` exists in the
  generated Admin TypeScript contract, but Admin Web does not expose it.
- Public library detail/source routes exist, but Admin Web does not consume
  them as a route-owned source inventory.
- Existing scan and NFO routes exist outside Admin Web, but UI mutation policy,
  confirmation, fallback behavior, and safe command results are not designed.
- Admin Web has no UI localization boundary; visible copy is hard-coded English.

## Target State

When this lane closes:

- `/libraries/:libraryId` is a route-owned Media Library management entry point.
- The route shows safe library facts, source/inventory state, metadata profile
  summary, scan/NFO readiness, and relevant jobs or follow-on links.
- Existing metadata-profile read/update semantics are represented safely in the
  Admin Web data-source boundary.
- Scan/NFO actions are either implemented with clear confirmation and result
  states or explicitly split with a documented API/UX blocker.
- Admin Web has a small localization boundary with English and Simplified
  Chinese message catalogs, and at least the app shell plus library management
  route use message IDs rather than hard-coded visible copy.
- Remaining Jellyfin/Plex parity gaps are split into narrower follow-ons:
  media browsing/detail, users/library access, settings mutation, playback
  runtime controls, and artwork management.

## In Scope

- Admin Web routes under `/libraries`.
- Library detail route and navigation from the library list.
- Metadata profile read/edit UX if it can reuse the accepted Admin API
  replacement semantics safely.
- Source inventory through existing public read routes or a documented Admin API
  bridge decision.
- Scan/NFO action UX policy and first safe command wiring if existing routes are
  adequate.
- Admin Web localization foundation for product UI copy.
- Tests, browser smoke, and redaction checks for touched routes.

## Out Of Scope

- Full media playback or watch-first UI in Admin Web.
- Full catalog/poster-wall browsing beyond administration-supporting links.
- User accounts, RBAC, or Library Access management.
- Runtime library create/delete if configuration authority is not accepted.
- Broad settings mutation outside library metadata/profile scope.
- Translating every Admin Web route in one pass.
- Addon Hosted Page embedding as trusted admin UI.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Library detail should be the next operator-visible slice. | High | `/libraries` closeout split detail, profile, scan/NFO, and inventory follow-ons. | Reprioritize toward Settings or Catalog only if library APIs are blocked. |
| Admin Web can reuse public read routes for library/source inventory. | Medium | Admin API matrix lists public library detail/sources as console-supporting reads. | Add or design a dedicated Admin library read model before source inventory UI. |
| Metadata profile update semantics are ready enough for UI design. | High | `admin-library-metadata-profile-configuration` closed with GET/PUT routes and TS contract. | Keep profile summary read-only and split field-specific PATCH commands. |
| Full UI localization should start with message IDs and route-local migration. | High | Admin Web has no i18n dependency or locale catalog. | Later translation work becomes a broad hard-coded-string cleanup. |

## Architecture Direction

Keep Admin Web feature-first:

- `adminApi/` owns generated Admin API contract consumption, public read-route
  bridges, redaction-preserving mappings, and route data-source methods.
- `features/libraries/` owns library list/detail/profile/source-management UI.
- `i18n/` owns locale choice, message catalogs, and small formatting helpers.
- Existing shared components remain neutral and accept rendered strings from the
  feature layer.

Do not mix the Public Client API and Admin API contract accidentally. If Admin
Web consumes public library/source routes, name that bridge explicitly and keep
admin-only mutation behavior behind Admin API methods.

## Closeout Condition

This lane can close when:

- the chosen `/libraries/:libraryId` management slice is implemented or split
  with a precise blocker,
- the Admin Web localization foundation is present and used by the new library
  management surface,
- targeted TypeScript, React, data-source, and browser-smoke gates pass,
- redaction tests cover unsafe library/source/profile text,
- `WORKSTREAM.json`, `TODO.md`, `EVIDENCE_AND_GATES.md`, and `HANDOFF.md` are
  updated with final evidence,
- and remaining parity gaps are split instead of hidden.
