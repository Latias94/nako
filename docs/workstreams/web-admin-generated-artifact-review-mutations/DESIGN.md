# Web Admin Generated Artifact Review Mutations - Design

Status: Closed
Last updated: 2026-05-29

## Problem

`web-admin-generated-artifacts-automation` shipped a redacted, read-only
Generated Artifact proposal queue. Operators still cannot complete the bounded
Acceptance Workflow from the new `web/` Admin shell, even though the backend
already exposes review-plan and review routes.

The missing slice must not become a generic AI assistant, raw payload preview,
or unguarded mutation button. A Generated Artifact remains an untrusted
automation proposal until an operator sees the review plan, understands the
write boundaries, and confirms accept or reject.

Closeout result: this lane shipped the guarded one-artifact review workflow in
the new `web/` Admin shell. Bulk review, Metadata Authority apply, provider
adapter breadth, and local automation runtime integration remain follow-ons.

## Target State

- `/admin/automation/generated-artifacts` stays the queue route.
- Queue rows provide guarded accept/reject entry points only when the proposal
  is actionable.
- `/admin/automation/generated-artifacts/review` owns
  `artifact_id` and `decision` search state.
- The review route loads a redacted `AdminGeneratedArtifactAcceptancePlan`
  before any mutation can run.
- The operator sees decision, status, action, reasons, readiness, target,
  payload summary, and all boundary flags.
- Confirmation is explicit; fixture mode remains truthfully non-persistent.
- Mutation results show `artifact_status`, `accepted_at`, and
  `idempotent_replay`.
- Successful review invalidates the proposal list and review-plan queries.

## Scope

In scope:

- `web/src/api/admin/client.ts`
- `web/src/api/admin/read-models-data-source.ts`
- `web/src/api/admin/mutations-data-source.ts`
- `web/src/features/admin/*`
- `web/src/shell/nako-router.tsx`
- `web/src/test/data-source-contracts.test.ts`
- `web/src/test/route-contracts.test.tsx`
- `web/src/test/route-state-contracts.test.tsx`
- this workstream's docs and architecture/workstream indexes

Out of scope:

- Backend route or schema changes.
- Metadata Authority apply.
- Raw prompt, raw payload body, provider raw response, local file path, Source
  Locator, credential, bearer token, secret, or storage-handle display.
- Bulk review.
- Provider adapter breadth or local automation runtime integration.

## Refactor Brief

**Intent**: remove the current product gap where generated automation proposals
are inspectable but cannot be safely reviewed in the future `web/`/Tauri
frontend.

**Scope**: Admin frontend API client, read-model data source, mutation data
source, Admin routes, and generated artifact UI surfaces.

**Deletion plan**: delete no still-used route. Remove stale read-only copy that
claims the route has no actions once guarded review controls land. Keep the old
`apps/admin-web` implementation as prior art only and do not copy it.

**Boundary plan**: keep review-plan mapping in the Admin read-model data
source, keep review execution in the Admin mutation data source, and keep URL
state in the TanStack router instead of local hidden state.

**Testing plan**: data-source contract tests for request serialization and
redaction; route contract tests for the review route; route-state tests for
queue-to-review navigation, fixture disabled mutation, live mutation, cache
invalidation, and unsafe-field suppression; TypeScript check; bundle budget;
desktop and mobile browser smoke.

**Risk plan**: the backend review-plan route is a `POST` route despite being a
planning read. The frontend must model that accurately. Mutation buttons stay
disabled when fixture mode or non-actionable plans would make the confirmation
misleading.

**Workflow plan**: durable workstream with `WGAR` tasks, closeout evidence, and
one precise Conventional Commit when verified.

## Architecture Direction

Use the generated Admin contract as the source of truth. The UI sees only a
small, redacted read model:

- IDs and enum-like statuses remain visible.
- Payload is represented by shape, fingerprint, byte/count facts, and
  confidence.
- Target is represented by kind and stable IDs.
- Boundary flags are rendered as policy facts, not implied writes.

Review mutation is not a generic `AdminMutationResult` because operators need
the domain-specific result: artifact status, idempotency replay, accepted time,
and the plan that was applied.

## Assumptions

- `crates/nako-server/src/http/admin.rs` remains authoritative for HTTP method
  and request body shape.
- The generated TypeScript contract continues to include
  `AdminGeneratedArtifactReviewRequest`,
  `AdminGeneratedArtifactReviewPlanResponse`, and
  `AdminGeneratedArtifactReviewResponse`.
- Fixture reads may show deterministic review-plan data, but fixture
  mutations must reject.

## Risks

| Risk | Mitigation |
| --- | --- |
| UI implies acceptance mutates canonical metadata immediately. | Render boundary flags explicitly and keep confirmation copy tied to those facts. |
| Review-plan route is treated as `GET`. | Data-source tests assert `POST` with `{ decision }`. |
| Unsafe generated content leaks through mapping. | Tests inject extra unsafe fields and assert they never render or serialize into read models. |
| Mutation leaves the queue stale. | Route tests assert proposal/review query invalidation after success. |
| Mobile review layout overflows. | Browser smoke covers desktop and mobile with horizontal overflow checks. |
