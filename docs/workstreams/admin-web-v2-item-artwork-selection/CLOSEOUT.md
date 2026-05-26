# Admin Web V2 Item Artwork Selection Closeout

Status: Closed
Closed: 2026-05-25

## Closeout Claim

This lane is complete. Admin Web V2 now supports an item-scoped Managed
Artwork gallery and guarded Selected Artwork select/replace and unpublish
workflow from `/items/:itemId`.

The lane does not claim candidate accept, ingest processing or requeue,
artifact lifecycle cleanup, storage drift/remediation, provider search,
uploads, catalog repair, Generated Artifact review, NFO writes, settings
mutation, users/permissions/Library Access, or full-site i18n.

## Delivered

- `/items/:itemId/artwork` with route-owned pagination state.
- A clear `/items/:itemId` support link into the artwork gallery.
- Generated Admin Web contract coverage for item artwork gallery, select, and
  unpublish routes.
- Explicit `AdminApiClient` methods for gallery GET, select POST, and
  unpublish DELETE with encoded item/artifact IDs and image kinds.
- `AdminDataSource` safe projections for candidate, artifact, Selected
  Artwork, and mutation-result summaries.
- Deterministic fallback for gallery reads only.
- Live-only select/unpublish mutation wrappers with visible failures and no
  fake mutation success fallback.
- Explicit prepare/confirm controls before select/replace or unpublish calls.
- Redaction-safe result rendering using first-party `/images/...` route paths
  only.
- Focused route, client, data-source, fallback, mutation, and redaction tests.
- Full Admin Web gate, focused Admin contract gate, whitespace gate, and
  desktop/mobile browser smoke evidence.

## Review Result

### Workstream Compliance

- Blocking: none.
- Important: none.
- `DESIGN.md` target state is satisfied.
- `TODO.md` tasks AWA-010 through AWA-070 are complete.
- ADR 0027 is respected: Admin-only artwork routes stay under `/admin/v1/*`
  and the UI does not expand the Public Client API.
- Managed Artwork lifecycle, remediation, provider search, upload, NFO,
  settings, users/permissions, and full-site i18n breadth remain split.

### Code Quality

- Blocking: none.
- Important: none.
- Admin Web API access remains behind `AdminApiClient` and `AdminDataSource`.
- The gallery route separates read fallback from live-only mutations.
- Mutation failures are visible and are not converted into mock successes.
- UI rendering accepts only already-redacted summaries and filters image refs
  to first-party `/images/...` paths.
- Tests exercise route behavior through public UI seams and API/data-source
  seams.

### Missing Gates

- None for this lane's target state.
- AWA-070 did not rerun browser smoke because it changed only closeout docs.
  The AWA-060 Playwright CLI smoke remains the official runtime evidence for
  item detail, artwork gallery, select confirmation, unpublish confirmation,
  desktop/mobile overflow, console errors, and unsafe artwork text exclusions.

## Follow-Ons

Recommended next lane:

1. `admin-web-v2-catalog-repair-actions`

Alternative product-priority lanes:

2. `admin-web-v2-settings-mutation`
3. `admin-web-v2-users-permissions-library-access`

Additional bounded follow-ons:

4. `admin-web-v2-metadata-diagnostics-read-model`
5. `admin-web-v2-item-nfo-status-actions`
6. `admin-web-v2-playback-support-detail`
7. `admin-web-v2-full-site-i18n`
8. Managed Artwork lifecycle/remediation/provider-search/upload breadth if
   operators need those controls in Admin Web.

## Evidence Anchors

- `docs/workstreams/admin-web-v2-item-artwork-selection/EVIDENCE_AND_GATES.md`
- `docs/workstreams/admin-web-v2-item-artwork-selection/ROUTE_API_READINESS.md`
- `apps/admin-web/src/features/items/ItemDetailPage.tsx`
- `apps/admin-web/src/features/items/ItemArtworkGalleryPage.tsx`
- `apps/admin-web/src/adminApi/client.ts`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/adminApi/generated/contract.ts`
- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/App.test.tsx`
- `apps/admin-web/src/adminApi/client.test.ts`
- `apps/admin-web/src/adminApi/dataSource.test.ts`
