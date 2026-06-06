# Current Functionality And Architecture Audit

## Goal

Audit Nako's current functional and architectural state after the M1 release
convergence work, then identify the next best tasks for Product-Operator M1 and
the deeper architecture risks that should not be allowed to harden.

## Scope

- Assess current state against the M1 Product-Operator journey: configure one
  Media Library, scan/index, browse catalog/media entries, play video, and use
  Admin diagnostics or repair surfaces.
- Review architecture map alignment across `docs/ROADMAP.md`,
  `docs/GOALS.md`, `docs/architecture/LANES.md`, and the relevant deep dives.
- Identify work that is already shipped so it is not reopened accidentally.
- Recommend a ranked next-work queue split between product functionality and
  architecture deepening.

## Current State Summary

Nako is no longer missing core backend primitives for the M1 journey. The recent
state is better described as "backend-capable, product-flow incomplete":

- M1 operator smoke exists through `scripts/m1-operator-journey-smoke.ps1` and
  composes server, Admin Web, and docs-safe release gates.
- Scan-originated Source Fingerprint hash triggering is implemented through the
  existing durable job path and disk-scan scheduler.
- Source Duplicate Relationship planning and apply are implemented at the
  core/server/Admin API level, with generated Admin contract types.
- Playback has a focused release gate mode, host hardware report evidence, and
  Media Web first-watch hardening already shipped.
- Route, contract, redaction, and diagnostics discipline is mature across many
  backend slices.

The main M1 risk is not raw capability. It is operator continuity: some shipped
backend capabilities are not yet connected into the Admin Web journey or the
release ladder in a way a real operator can discover and execute.

## Findings

### High / P1

#### 1. Source Duplicate reconciliation is backend-complete but not operator-visible

Evidence:

- `crates/nako-api/src/admin_contract.rs` includes
  `sourceDuplicateReconciliationPlan` and
  `sourceDuplicateReconciliationApply`.
- `crates/nako-server/src/app/source_duplicate.rs` implements plan/apply
  behavior with redaction and stale guards.
- `apps/admin-web/src/adminApi/generated/contract.ts` has generated DTOs and
  routes.
- `rg` found no non-generated Admin Web feature usage for the duplicate
  reconciliation plan/apply routes.

Impact:

- Operators cannot complete the M1 "diagnose or repair common failures" story
  for duplicate Media Sources from the UI, even though the backend path is
  available.
- The capability may regress unnoticed because the M1 smoke currently maps
  diagnostics/repair broadly, not this specific operator repair flow.

Recommended next task:

- `source-duplicate-reconciliation-operator-flow`
- Add an Admin Web flow from item/source inventory into the duplicate
  reconciliation plan, render redacted candidates/actions, and call apply only
  for `suggest_relationship` with explicit confirmation.

#### 2. Roadmap and active queue need a post-completion refresh

Evidence:

- `docs/ROADMAP.md` still lists the M1 queue items, but several are already
  archived as completed Trellis tasks:
  - `m1-operator-journey-smoke`
  - `scan-originated-source-hash-triggering`
  - source duplicate plan/apply backend slices
  - playback release gate mode and Media Web playback hardening slices
- `docs/architecture/LANES.md` still names the source duplicate operator flow,
  Media Web browse/player smoke, and release ladder runner as candidates, but
  does not distinguish backend-complete versus UI/release-runner remaining
  work.

Impact:

- Future sessions may reopen completed backend slices or choose a lower-value
  task because the queue is not synchronized with evidence.

Recommended next task:

- `m1-roadmap-queue-refresh-after-source-hash-and-duplicate-backend`
- Docs-only: update `ROADMAP.md`, `GOALS.md`, and `LANES.md` to mark completed
  slices and define the next three executable tasks.

### Medium / P2

#### 3. M1 smoke is deterministic but not a live operator run

Evidence:

- `scripts/m1-operator-journey-smoke.ps1` composes existing deterministic tests.
- The archived task explicitly records that it does not prove an actual live
  operator browser session against a running self-hosted instance.

Impact:

- This is acceptable for the first smoke slice, but M1 release confidence will
  need a script or runbook that starts Nako, serves Admin Web/Media Web, and
  drives a real configured-library path.

Recommended next task:

- `m1-release-ladder-runner`
- Build a staged runner that can execute docs-safe, server, Admin Web, playback,
  container/config, and optional live-browser checks without requiring all
  expensive gates on every local run.

#### 4. Admin Web data source interface is becoming a shallow pass-through surface

Evidence:

- `apps/admin-web/src/adminApi/dataSource.ts` is broad and mixes loading,
  fallback behavior, DTO mapping, and feature-specific command methods.
- Newly generated duplicate reconciliation routes are not surfaced in
  `AdminDataSource`, which made the UI gap easy to miss.

Impact:

- Each new Admin capability requires remembering to update client, data source,
  mock data, route UI, i18n, and tests separately. This is manageable now, but
  the Interface is widening faster than product routes are converging.

Architecture opportunity:

- Deepen feature-owned Admin data modules for high-churn workflows. For source
  duplicate reconciliation, create a feature-owned data adapter that hides
  generated route details and redaction/fallback mapping behind a small
  Interface used by the UI and tests.

#### 5. Large backend test files are carrying a lot of product confidence

Evidence:

- Largest files include `crates/nako-server/src/http/tests/addons.rs`,
  `crates/nako-server/src/http/tests/system.rs`,
  `crates/nako-db/src/contract_tests.rs`, and several large app test modules.

Impact:

- The size is not automatically wrong: these tests encode valuable redaction
  and contract behavior. The risk is navigation and change-locality cost.

Architecture opportunity:

- Do not split mechanically. Instead, when touching one route family, extract
  route-family fixture builders and assertion helpers if deletion would
  otherwise scatter complexity across new tests.

## Recommended Next Work Queue

### 1. Product P1: Source Duplicate reconciliation operator flow

Outcome:

- Admin Web exposes the existing source duplicate plan/apply routes from item
  or library source inventory.
- Operators see redacted candidate facts, existing relationship state,
  recommended action, and explicit confirmation before mutation.
- M1 smoke gains a focused assertion that the repair flow is reachable and does
  not leak locators, paths, hashes, or fingerprints.

Why first:

- It turns an already-shipped backend capability into an operator-visible M1
  repair flow with limited new backend risk.

### 2. Planning P1: Refresh roadmap and lane queue after completed slices

Outcome:

- `ROADMAP.md`, `GOALS.md`, and `LANES.md` reflect that M1 smoke,
  scan-originated hash triggering, source duplicate backend plan/apply, and
  playback release gates are complete.
- The next queue becomes concrete:
  1. source duplicate Admin Web operator flow;
  2. M1 release ladder runner/live smoke;
  3. Media Web browse/player polish only if current smoke exposes a blocker.

Why second:

- Prevents agents from repeating already-landed backend work.

### 3. Release P2: M1 release ladder runner

Outcome:

- One documented runner or matrix drives `docs`, `server`, `admin-web`,
  `playback`, `container`, `postgres`, and optional live-browser modes.
- The runner records evidence locations and makes expensive gates explicit.

Why third:

- M1 should be easy to validate repeatedly before adding more breadth.

### 4. Architecture P2: Admin Web feature data adapter deepening

Outcome:

- Start with source duplicate reconciliation and avoid broad frontend
  refactor.
- Feature UI depends on a small adapter Interface, not the entire generated
  contract and broad `AdminDataSource` surface.

Why fourth:

- This improves locality exactly where new M1 operator flows will keep landing.

### 5. Architecture P2: Test fixture locality for route families

Outcome:

- When the next backend route family is touched, move only the relevant setup
  and redaction assertions into family-owned helpers.
- Avoid a generic test framework rewrite.

Why fifth:

- Test size is a maintainability pressure, not a product blocker.

## Out Of Scope

- No code changes are required by this audit task.
- No new schema, API, generated contract, or runtime behavior changes.
- No Public Client metadata governance expansion.
- No Addon Manager work.
- No playback codec/HDR/device-profile breadth unless M1 smoke reveals a
  concrete playback blocker.

## Acceptance Criteria

- [x] Current git state and push status checked.
- [x] M1 roadmap, goals, lane routing, and architecture maps reviewed.
- [x] Recent completed Trellis evidence checked for M1 smoke, source hash,
      source duplicate, Media Web playback, and release gate work.
- [x] Product/architecture findings are ranked by priority.
- [x] Next-work queue is concrete enough to open focused Trellis tasks.

## Technical Notes

Inspected key files:

- `docs/ROADMAP.md`
- `docs/GOALS.md`
- `docs/ARCHITECTURE.md`
- `docs/architecture/LANES.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/OPERATIONS_RELEASE.md`
- `CONTEXT.md`
- `scripts/m1-operator-journey-smoke.ps1`
- `scripts/release-gate.ps1`
- archived M1/source-hash/source-duplicate/playback evidence under
  `.trellis/tasks/archive/2026-06/`
- Admin Web data source and item/library detail pages.

Push result:

- `git push origin main` completed successfully before this audit.
