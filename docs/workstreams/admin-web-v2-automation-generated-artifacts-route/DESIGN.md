# Admin Web V2 Automation Generated Artifacts Route

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

The legacy dashboard still owns the Generated Artifacts panel. This workflow is
where operators inspect AI-assisted proposals before any accepted authority or
sidecar-writing flow exists. The generated Admin API already exposes
`GET /admin/v1/automation/generated-artifacts/proposals`, so the read-only
proposal list can move to a route-first V2 page without backend changes.

## Relevant Authority

- `PRODUCT.md`
- `DESIGN.md`
- `CONTEXT.md`
- `docs/workstreams/admin-web-v2-product-architecture/ROUTE_API_READINESS.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`

## Problem

Operators cannot inspect generated artifact proposals from V2 navigation
without returning to the legacy anchor page. The legacy panel does not own URL
pagination state, route-local fallback, or focused tests proving that prompt and
payload content stay reduced to fingerprints and safe summary fields.

## Target State

When this lane closes:

- `/automation/generated-artifacts` is a real route-first V2 page.
- The route owns `limit` and `offset` search params from
  `AdminGeneratedArtifactProposalsQuery`.
- `AdminDataSource` exposes route-local live/mock fallback for
  `AdminGeneratedArtifactProposalListResponse`.
- The page renders proposal capability, kind, target summary, readiness,
  payload shape, confidence, provider name, attempt count, and fingerprints
  without prompt text, payload bodies, local paths, raw provider data, or
  credentials.
- The route has focused tests, redaction checks, frontend gates, and browser
  smoke evidence.

## In Scope

- Route wiring and navigation entry for `/automation/generated-artifacts`.
- Route-owned pagination search params and generated query DTO mapping.
- Read-only V2 table/panel UI for proposal review readiness.
- Route-local `AdminDataSource.loadGeneratedArtifacts`.
- Tests for rendering, fallback, search params, and unsafe text exclusions.
- Workstream evidence and closeout updates.

## Out Of Scope

- Accept/reject mutation UI.
- Review-plan detail routes.
- Automation events route.
- Backend/Admin API contract changes.
- Rendering prompt text, payload bodies, raw provider responses, credentials,
  local paths, or source locators.
- Removing `/legacy`.

## Architecture Direction

Follow the existing route pattern used by Jobs, Catalog Governance, Playback
Sessions, Storage Staging, and Addons: `App.tsx` owns route wiring and URL
normalization, `adminApi/dataSource.ts` owns live/mock fallback, and
`features/automation` owns display-only workflow UI.

## Closeout Condition

The lane can close when `/automation/generated-artifacts` has route-first
behavior, validation evidence, browser smoke evidence, and explicit follow-ons
for review-plan and accept/reject workflows.

Status: satisfied on 2026-05-25. See `CLOSEOUT.md`.
