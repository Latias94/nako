# Admin Web V2 Media Browsing And Item Detail Governance Closeout

Status: Closed
Closed: 2026-05-25

## Closeout Claim

This lane is complete. Admin Web V2 now has route-owned, governance-oriented
`/catalog` browse/search and `/items/:itemId` detail workflows backed by
explicit public-read bridge methods and safe Admin Web route summaries.

The lane does not claim repair/action parity. Catalog repair, Generated
Artifact review/apply, item artwork selection, safe metadata diagnostics,
NFO item status/actions, playback support detail, settings mutation,
users/permissions/Library Access, and full-site i18n remain follow-ons.

## Delivered

- `/catalog` browse/search with URL-owned `q`, `facet`, `limit`, and `offset`.
- `/items/:itemId` governance detail with Media Item facts, Canonical Metadata
  summary, safe Media Source filenames, bounded source probe summaries, public
  image readiness, split-workflow readiness placeholders, and support links.
- Explicit Admin Web public-read bridge methods for catalog browse/search, item
  detail, item credits/images, and source probes.
- Safe route-local projections that do not render Source Locators, local paths,
  raw provider payloads, artifact storage handles, playback output paths,
  tokens, or secret-like values.
- Deterministic fallback behavior.
- Route, bridge, data-source, fallback, source-probe limit, and redaction tests.
- Desktop and mobile browser smoke evidence for `/catalog` and
  `/items/item-unknown-1`.
- `FOLLOW_ON_SPLIT.md` with bounded next lanes and recommended order.

## Review Result

### Workstream Compliance

- Blocking: none.
- Important: none.
- `DESIGN.md` target state is satisfied.
- `TODO.md` tasks MBG-010 through MBG-060 are complete.
- ADR 0027 is respected: public reads are explicitly named bridges, and
  admin-only diagnostics/mutations stay out of Public Client API surfaces.
- Repair/action breadth is split rather than hidden inside the read slice.

### Code Quality

- Blocking: none.
- Important: none.
- Admin Web API access remains behind `AdminApiClient` and `AdminDataSource`.
- UI routes own their URL/query state and render only safe summaries.
- Tests exercise behavior through route/data-source/client seams.
- Browser smoke confirms nonblank routes, no horizontal overflow, no console
  errors, and no unsafe rendered text in fallback paths.

### Missing Gates

- None for this lane's target state.
- Rust nextest was not rerun for MBG-060 because closeout added only Admin Web
  frontend/doc evidence after already-verified Rust/Admin API work from the
  prior library-management lane. Admin Web gates and `git diff --check` are the
  relevant closeout gates for this lane.

## Follow-Ons

Recommended next lane:

1. `admin-web-v2-generated-artifact-review-actions`

Additional split lanes:

2. `admin-web-v2-item-artwork-selection`
3. `admin-web-v2-catalog-repair-actions`
4. `admin-web-v2-metadata-diagnostics-read-model`
5. `admin-web-v2-item-nfo-status-actions`
6. `admin-web-v2-playback-support-detail`

Broader Admin Web V2 backlog, outside this lane:

- settings mutation;
- user, role, permission, and Library Access management;
- full-site i18n expansion.

## Evidence Anchors

- `docs/workstreams/admin-web-v2-media-browsing-and-item-detail-governance/EVIDENCE_AND_GATES.md`
- `docs/workstreams/admin-web-v2-media-browsing-and-item-detail-governance/FOLLOW_ON_SPLIT.md`
- `apps/admin-web/src/features/catalog/CatalogBrowsePage.tsx`
- `apps/admin-web/src/features/items/ItemDetailPage.tsx`
- `apps/admin-web/src/adminApi/client.ts`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/App.test.tsx`
- `apps/admin-web/src/adminApi/client.test.ts`
- `apps/admin-web/src/adminApi/dataSource.test.ts`
