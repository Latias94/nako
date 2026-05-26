# Admin Web V2 Library Management And Localization Closeout

Status: Closed
Closed: 2026-05-25

## Closeout Claim

This lane is complete. Admin Web V2 now has a route-owned Media Library detail
management workflow, explicit Metadata Profile full-replacement editing,
confirmed scan/NFO command actions through Admin API wrappers, a safe Source
inventory bridge summary, and the first app-shell/library-management i18n
boundary.

The lane also re-scored remaining Admin Web V2 management parity gaps and split
them into bounded follow-ons. It does not claim that all Admin Web management
parity is complete.

## Delivered

- `/libraries/:libraryId` route wiring and navigation from `/libraries`.
- Library detail panels for safe library facts, Metadata Profile, Source
  inventory, and operations.
- Metadata Profile GET/PUT editing UX with explicit full-replacement copy.
- Admin API command wrappers:
  - `POST /admin/v1/libraries/{library_id}/scan`
  - `POST /admin/v1/libraries/{library_id}/nfo/import`
  - `POST /admin/v1/libraries/{library_id}/nfo/export`
- Generated Admin Web TypeScript contract updates for library commands and
  redaction-safe job command responses.
- Admin Web data-source methods for library detail, profile replacement,
  scan/NFO command enqueue, and public-read Source inventory bridge summaries.
- English and Simplified Chinese message catalogs, shell locale selector, and
  localized app shell plus library-management visible copy.
- Redaction tests for unsafe local paths, Source Locators, secret-like fields,
  tokens, and raw provider/source payload text.
- `PARITY_GAP_SPLIT.md` with the next bounded follow-on recommendation:
  `admin-web-v2-media-browsing-and-item-detail-governance`.

## Review Result

### Workstream Compliance

- Blocking: none.
- Important: none.
- AWL-010, AWL-020, AWL-030, AWL-040, AWL-050, and AWL-060 are complete.
- The `DESIGN.md` target state is satisfied for this lane's scope.
- Remaining Jellyfin/Plex-style expectations are split into follow-ons instead
  of hidden in this lane.
- Admin Web remains administration-first and does not become a playback client.

### Code Quality

- Blocking: none.
- Important: none.
- Admin-only scan/NFO mutations use `/admin/v1/*` wrappers, preserving ADR 0027.
- Public Client source reads are consumed only through an explicitly named
  bridge and mapped to safe summaries before UI rendering.
- Metadata Profile editing keeps full PUT replacement semantics visible.
- Source inventory job summaries filter by `library_id` before counting failed
  jobs, avoiding cross-library hybrid/mock leakage.
- Tests exercise public seams: route rendering, Admin API client paths,
  data-source mapping, command confirmation, and redaction behavior.

### Missing Gates

- None for this lane's target state.
- The broader workspace nextest suite was not run because this lane touched
  focused Admin Web, Admin API contract, and library HTTP route surfaces; the
  recorded focused gates cover those changed surfaces.

## Follow-Ons

Recommended next lane:

1. `admin-web-v2-media-browsing-and-item-detail-governance`
   - Add `/catalog` and `/items/:itemId` as governance-oriented browse/detail
     routes.
   - Keep playback and watch-state out of scope.
   - Use explicit public-read bridges and redaction tests.

Other split candidates:

2. `admin-web-v2-settings-and-network-mutation-authority`
3. `admin-web-v2-users-roles-library-access`
4. `admin-web-v2-governance-repair-actions`
5. `admin-web-v2-addon-operations-mutations`
6. `admin-web-v2-playback-support-detail`
7. `admin-web-v2-i18n-expansion`

## Residual Risks

- Metadata Profile editing is intentionally full replacement. Field-specific
  patching remains a follow-on if operators need safer partial updates.
- Source inventory currently bridges public read routes rather than owning a
  dedicated Admin library inventory read model.
- Settings mutation, users/access, repair/apply workflows, and broader i18n are
  explicitly outside this lane.
- Vite still emits the existing large app-bundle chunk warning.

## Evidence Anchors

- `docs/workstreams/admin-web-v2-library-management-and-localization/EVIDENCE_AND_GATES.md`
- `docs/workstreams/admin-web-v2-library-management-and-localization/PARITY_GAP_SPLIT.md`
- `apps/admin-web/src/features/libraries/LibraryDetailPage.tsx`
- `apps/admin-web/src/features/libraries/LibrariesPage.tsx`
- `apps/admin-web/src/adminApi/client.ts`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/i18n/`
- `crates/nako-api/src/admin_contract.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/tests/library.rs`
