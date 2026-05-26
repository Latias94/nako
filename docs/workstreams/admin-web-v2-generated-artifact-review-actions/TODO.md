# Admin Web V2 Generated Artifact Review Actions - TODO

Status: Closed
Last updated: 2026-05-25

Task IDs use the `GAR` prefix.

## M0 - Scope And Evidence Freeze

- [x] GAR-010 [owner=codex] [deps=none] [scope=docs/workstreams/admin-web-v2-generated-artifact-review-actions,docs/workstreams/README.md]
  Goal: Open the lane, freeze scope, non-goals, review/action order, validation
  gates, and first executable task.
  Validation: Workstream docs exist and agree with the closed Generated
  Artifacts read-only route and MBG-050 follow-on split.
  Review: This lane must not absorb catalog repair, artwork, NFO, Provider
  Mapping, or full-site i18n work.
  Evidence: `DESIGN.md`, `TODO.md`, `MILESTONES.md`,
  `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Result: DONE 2026-05-25. Lane opened from MBG-050 closeout recommendation.
  Handoff: Continue with GAR-020 route/API readiness.

## M1 - Review Route/API Readiness

- [x] GAR-020 [owner=codex] [deps=GAR-010] [scope=docs/workstreams/admin-web-v2-generated-artifact-review-actions,apps/admin-web/src/adminApi,docs/api/HTTP_API.md,crates/nako-api/src/admin/automation.rs,crates/nako-server/src/http/admin.rs]
  Goal: Audit generated Admin API review-plan/review route shapes, request
  bodies, result redaction, fallback needs, and route UX shape before UI
  implementation.
  Validation: `git diff --check`.
  Review: If review DTOs expose unsafe payload/raw/provider data or mutation
  semantics are unclear, split backend/API hardening before UI.
  Evidence: readiness notes and updated handoff/evidence.
  Result: DONE 2026-05-25. `ROUTE_API_READINESS.md` accepts the generated
  Admin review-plan and review routes for one-proposal review, documents safe
  projection rules, forbids fake successful mutation fallback, and updates
  `docs/api/HTTP_API.md` route inventory.
  Handoff: Continue with GAR-030 review-plan UI.

## M2 - Review Plan UI

- [x] GAR-030 [owner=frontend] [deps=GAR-020] [scope=apps/admin-web/src/App.tsx,apps/admin-web/src/features/automation,apps/admin-web/src/adminApi,apps/admin-web/src/App.test.tsx,apps/admin-web/src/adminApi/client.test.ts,apps/admin-web/src/adminApi/dataSource.test.ts]
  Goal: Add one-proposal review plan UI reachable from
  `/automation/generated-artifacts`, with safe plan summary, decision selection,
  deterministic fallback, and redaction tests.
  Validation: `cd apps/admin-web && npm run check && npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts`.
  Review: Do not post mutations from the plan view without explicit
  confirmation.
  Evidence: route, client, data-source, fallback, and redaction tests.
  Result: DONE 2026-05-25. Added `/automation/generated-artifacts/$artifactId/review`
  with route-local `?decision=accept|reject`, safe review-plan projection,
  deterministic plan fallback, list-to-review navigation, and redaction tests.
  No accept/reject mutation is posted from the plan view.
  Handoff: Continue with GAR-040 confirmation/mutation.

## M3 - Confirmed Review Action

- [x] GAR-040 [owner=frontend] [deps=GAR-030] [scope=apps/admin-web/src/features/automation,apps/admin-web/src/adminApi,apps/admin-web/src/App.test.tsx,apps/admin-web/src/adminApi/client.test.ts,apps/admin-web/src/adminApi/dataSource.test.ts]
  Goal: Add explicit accept/reject confirmation and redacted result rendering
  for one Generated Artifact proposal.
  Validation: `cd apps/admin-web && npm run check && npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts`.
  Review: Confirm mutation request body, idempotent/error behavior, route-local
  result state, and no unsafe text rendering.
  Evidence: mutation tests, confirmation tests, and redaction tests.
  Result: DONE 2026-05-25. Added explicit prepare/confirm review action state,
  real Admin API review mutation client/data-source wiring, redacted result
  rendering, visible mutation error behavior, and tests proving no fake
  successful mutation fallback.
  Handoff: Continue with GAR-050 verification.

## M4 - Verification And Browser Smoke

- [x] GAR-050 [owner=codex] [deps=GAR-040] [scope=apps/admin-web,docs/workstreams/admin-web-v2-generated-artifact-review-actions]
  Goal: Run focused/full Admin Web gates and browser smoke for the proposal
  list plus one review/confirmation path.
  Validation: `cd apps/admin-web && npm run check && npm run test && npm run build`; `git diff --check`; browser smoke.
  Review: Browser smoke must check desktop/mobile, no horizontal overflow, no
  console errors, and no unsafe rendered prompt/payload/provider/path/token
  text.
  Evidence: `EVIDENCE_AND_GATES.md`, browser smoke notes.
  Result: DONE 2026-05-25. Fresh full Admin Web gate, `git diff --check`,
  and browser smoke passed for proposal queue plus accept/reject confirmation
  paths at desktop and mobile widths.
  Handoff: Continue with GAR-060 closeout.

## M5 - Closeout

- [x] GAR-060 [owner=planner] [deps=GAR-050] [scope=docs/workstreams/admin-web-v2-generated-artifact-review-actions]
  Goal: Close the lane or split blockers and update status fields.
  Validation: final Admin Web gates, browser smoke evidence, and `git diff --check`.
  Review: `review-workstream` and `verify-rust-workstream` before completion
  claims.
  Evidence: `CLOSEOUT.md`, `WORKSTREAM.json`, `HANDOFF.md`,
  `EVIDENCE_AND_GATES.md`.
  Result: DONE 2026-05-25. Final Admin Web gates, closeout review,
  `git diff --check`, and desktop/mobile browser smoke passed. The lane is
  closed with remaining bulk review, catalog repair, artwork, NFO,
  metadata-diagnostics, settings mutation, users/permissions/Library Access,
  and full-site i18n breadth preserved as follow-ons.
  Handoff: Open `admin-web-v2-item-artwork-selection` next unless product
  priority pulls settings mutation or users/Library Access forward.
