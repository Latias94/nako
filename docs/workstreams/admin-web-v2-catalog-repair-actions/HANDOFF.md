# Admin Web V2 Catalog Repair Actions - Handoff

Status: Complete
Last updated: 2026-05-25

## Current State

The lane is complete. CRA-010 through CRA-070 are complete:

- The lane was opened from AWA closeout and the MBG follow-on map.
- Scope is bounded Catalog Governance repair, starting with route/API readiness.
- The current Admin Web route is read-only and backed by the existing
  `catalogGovernanceItems` Admin API route.
- Current generated Admin Web contract coverage is sufficient for the read-only
  queue, but not for repair detail, review-plan, or mutation.
- CRA-020 accepted Provider Mapping accept/reject as the first repair action
  because the core status vocabulary already has `candidate`, `accepted`, and
  `rejected`, and the action can be scoped to one Media Item plus one Provider
  Mapping.
- CRA-030 added a redaction-safe Admin API item detail route and Provider
  Mapping review-plan route, generated Admin Web contract coverage,
  `AdminApiClient` wrappers, HTTP API docs, and focused tests.
- CRA-040 added the confirmed Provider Mapping review mutation, idempotent
  changed/replay semantics, generated Admin Web contract coverage,
  `AdminApiClient` and `AdminDataSource` wrappers, HTTP API docs, and tests
  proving read fallback is deterministic while mutation fallback cannot fake
  success.
- CRA-050 added a route-owned Admin Web repair context at
  `/catalog/governance/{item_id}`, queue-to-review navigation, Provider Mapping
  selection, decision URL state, review-plan preview, explicit confirmation,
  result/failure states, and unsafe text exclusion tests.
- CRA-060 ran focused Rust/API/Admin Web verification and browser smoke for the
  queue, repair context, review-plan, confirmation success, visible failure,
  desktop/mobile overflow, console errors, and unsafe text exclusions.
- Unknown-item classification, hierarchy repair, duplicate-source merge/split,
  rematch/provider search, NFO writes, arbitrary metadata editing, settings,
  users/permissions/Library Access, and full-site i18n are out of scope until
  explicitly accepted or split.

## Active Task

- Task ID: CRA-070
- Owner: planner
- Files:
  - `docs/workstreams/admin-web-v2-catalog-repair-actions`
- Validation: final gates, browser smoke evidence, review, verify, closeout
  docs, and `git diff --check`.
- Status: READY
- Review: pending
- Evidence: CRA-060 focused gates and browser smoke passed.

## Decisions

- Do not add row-level repair buttons directly to `/catalog/governance`.
- Mutations require review-plan or dry-run context plus explicit confirmation.
- Gallery-style deterministic fallback is allowed for read context only.
- Repair mutations must not fake success when the live Admin API fails.
- Admin Web must not render raw Local Inference, Source Locator, path, provider
  body, provider URL, NFO, token, credential, or arbitrary metadata payload
  text.
- First action: Provider Mapping `accept` or `reject` for one `item_id` and one
  `mapping_id`.
- The action updates Provider Mapping status only. It must not mutate Canonical
  Metadata, Provider Subjects, Local Inference Evidence, Source Duplicate
  Relationships, hierarchy, NFO sidecars, Library Files, artwork, playback
  state, or Public Client API state.
- Proposed route shape for CRA-030/CRA-040:
  - `GET /admin/v1/catalog/governance/items/{item_id}`
  - `POST /admin/v1/catalog/governance/items/{item_id}/provider-mappings/{mapping_id}/review-plan`
  - `POST /admin/v1/catalog/governance/items/{item_id}/provider-mappings/{mapping_id}/review`
- Implemented in CRA-030:
  - `GET /admin/v1/catalog/governance/items/{item_id}`
  - `POST /admin/v1/catalog/governance/items/{item_id}/provider-mappings/{mapping_id}/review-plan`
  - generated contract constants/types for those routes
  - `AdminApiClient.getCatalogGovernanceItemDetail`
  - `AdminApiClient.planCatalogGovernanceProviderMappingReview`
- Implemented in CRA-040:
  - `POST /admin/v1/catalog/governance/items/{item_id}/provider-mappings/{mapping_id}/review`
  - generated contract constants/types for the mutation response
  - `AdminApiClient.reviewCatalogGovernanceProviderMapping`
  - `AdminDataSource.loadCatalogGovernanceItemDetail`
  - `AdminDataSource.loadCatalogGovernanceProviderMappingReviewPlan`
  - `AdminDataSource.reviewCatalogGovernanceProviderMapping`
- Implemented in CRA-050:
  - `/catalog/governance/{item_id}`
  - queue row review navigation
  - Provider Mapping selector
  - decision search state
  - review-plan preview
  - explicit prepare/confirm mutation flow
  - mutation result and unavailable-action error rendering
- Verified in CRA-060:
  - focused Admin Web route/client/data-source tests
  - focused Rust mutation/API contract tests
  - formatting and whitespace checks
  - desktop queue smoke
  - desktop repair context confirmation success smoke
  - visible failure smoke
  - mobile repair context smoke

## Blockers

- None for CRA-070.

## Next Recommended Action

Open the next Admin Web V2 lane for the remaining higher-breadth Catalog
Governance repair classes or follow-on admin experience work. The remaining
repair breadth is intentionally split and should not be reabsorbed into this
lane without a new scope decision.
