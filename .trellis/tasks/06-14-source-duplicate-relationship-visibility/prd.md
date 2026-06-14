# Source duplicate relationship visibility and filters

## Goal

Make reviewed Source Duplicate Relationships easier for an administrator to
find and scan after the suggestion review flow lands. Operators should be able
to distinguish pending suggestions, confirmed relationships, rejected
relationships, stale candidates, and refresh-needed candidates without exposing
Source Locators, raw fingerprints, local paths, tokens, or job payloads.

## What I Already Know

- The previous task shipped `suggest_relationship`, `confirm_suggested`, and
  `reject_suggested` on the existing Admin apply route.
- Existing Admin Web route:
  `/items/$itemId/sources/$sourceId/duplicates?library_id=...`.
- Existing route data already includes per-candidate `existing_status`,
  `relationship_id`, `recommended_action`, `stale`, `evidence_kind`, and
  confidence.
- `ItemDetailPage` already renders a "Review duplicates" link for every Media
  Source and a support link to the first source's duplicate review route.
- Catalog Governance item records already expose
  `duplicate_relationship_count`, but `ItemDetailSummary` does not currently
  include per-source duplicate relationship counts or status summaries.
- Source Duplicate Relationship remains evidence/review state. It must not
  merge Media Sources, change Playback Source Selection, or collapse Media
  Items.

## Assumptions

- MVP should keep using the existing reconciliation plan route unless we choose
  to expose broader item-level aggregate data.
- Admin Web mock fallback should remain deterministic and should not fabricate
  new write outcomes.
- Any new data exposed to Admin Web must remain redaction-safe and bounded.

## Selected MVP

Use **Approach A: Reconciliation Route Filters**.

This task improves the existing Admin Web duplicate reconciliation page only.
It does not change backend contracts, generated Admin API output, database
schema, or item detail summaries.

## Requirements

- Add URL-owned route-local filters to the existing duplicate reconciliation
  page: `status`, `action`, and `freshness`.
- Filter the current bounded candidate page client-side because the route
  already carries `existing_status`, `recommended_action`, and `stale`.
- Add quick filter buttons for Pending Suggestion, Suggested Review, Confirmed,
  Rejected, Refresh Needed, and Stale.
- Reset `offset` to `0` when filters change.
- Preserve existing live-only mutation behavior.
- Keep raw Source Locators, paths, fingerprints, hashes, tokens, provider
  payloads, and job input/summary JSON out of responses and rendered text.

## Acceptance Criteria

- [x] Confirmed and rejected relationships are visibly distinguishable from
      pending suggestions on the Admin duplicate review surface.
- [x] Operators can narrow the displayed relationships/candidates by status or
      review action.
- [x] Operators can apply common quick filters without typing URL params.
- [x] The existing plan load call still receives only `limit` and `offset`.
- [x] The UI keeps mutation controls live-only and does not fabricate mock
      confirm/reject outcomes.
- [x] Tests cover URL search normalization, i18n copy, filter behavior, and
      redaction of unsafe injected fields.
- [x] No API/DTO/schema change is introduced for this MVP.

## Definition Of Done

- Relevant Rust/Admin Web tests pass for changed layers.
- `cargo fmt --all` for Rust changes.
- Admin Web `check`, focused tests, and build pass when route code changes.
- Trellis task context and closeout evidence are updated.
- Specs are updated if a new API contract or repeatable UI workflow convention
  is introduced.

## Out Of Scope

- Automatic Media Source or Media Item merge.
- Playback Source Selection changes.
- Undo/reopen of Confirmed or Rejected relationships.
- Bulk confirm/reject.
- New DB schema or audit event table.
- Public Client API exposure.
- Item-detail aggregate duplicate status/count hints.
- Item-level duplicate relationship inventory route.
- Server-side pagination/filter semantics for relationship filters.

## Technical Notes

- Existing page: `apps/admin-web/src/features/items/SourceDuplicateReconciliationPage.tsx`
- Existing route shell:
  `apps/admin-web/src/features/items/SourceDuplicateReconciliationRoutePage.tsx`
- Existing item detail source links:
  `apps/admin-web/src/features/items/ItemDetailPage.tsx`
- Existing Admin DTOs:
  `crates/nako-api/src/admin/operations.rs`
- Existing app service:
  `crates/nako-server/src/app/source_duplicate.rs`
- Existing spec:
  `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`

## Implementation Summary

- Added URL-owned `status`, `action`, and `freshness` filters to the existing
  source duplicate reconciliation route.
- Added quick filters for Pending Suggestion, Suggested Review, Confirmed,
  Rejected, Refresh Needed, and Stale.
- Kept filtering client-side against the current bounded candidate page and
  preserved the existing plan request payload as `{ limit, offset }`.
- Added relationship status badges for `none`, `suggested`, `confirmed`, and
  `rejected`.
- Added English and zh-Hans copy and route tests for URL/search behavior,
  quick filters, i18n, live-only mutation preservation, and redaction.

## Verification Evidence

- `npm run check --prefix apps/admin-web`: passed.
- `npm run test --prefix apps/admin-web -- src/App.test.tsx`: passed, 138 tests.
- `npm run test --prefix apps/admin-web`: passed, 245 tests across 8 files.
- `npm run build --prefix apps/admin-web`: passed.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-14-source-duplicate-relationship-visibility`: passed.
- `git diff --check`: passed.
- `npm run format --prefix apps/admin-web -- --check`: not available; `apps/admin-web`
  has no `format` script.

## Spec Update Decision

No Trellis spec update was needed. This task follows the existing Admin Web
route-owned search/filter pattern and does not introduce a new API contract or
repeatable workflow convention.
