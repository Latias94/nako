# Generated Artifact Metadata Authority Apply - Handoff

Status: Closed
Last updated: 2026-06-01

## Current State

`GAMA-070` is complete and the lane is closed.
Planner reconciliation on 2026-06-01 reran the focused final backend route
gates, and the Web Admin apply workflow has now landed.

Generated Artifact review acceptance is still guarded, and the new read-only
metadata apply-plan route now exposes field-level, redacted plan facts without
mutating Canonical Metadata.

Accepted metadata Generated Artifacts now have a host-owned app-layer apply
execution path. The path rebuilds an executable plan, rejects stale targets
before mutation, applies through `MetadataApplication` with all field locks
protected, and commits Canonical Metadata plus catalog/search projection in one
metadata application transaction.

Generated Artifact metadata apply outcomes are now durable. Each apply request
uses an explicit idempotency key, persists an `applied`, `noop`, or `failed`
outcome, stores the redacted apply plan, and atomically commits Canonical
Metadata plus catalog/search projection with the outcome record for applied
mutations.

The final Admin apply route is exposed at
`POST /admin/v1/automation/generated-artifacts/{artifact_id}/metadata-apply`.
The request body is `AdminGeneratedArtifactMetadataApplyRequest` with an
explicit `idempotency_key`. The response is
`AdminGeneratedArtifactMetadataApplyResponse`, containing only redacted outcome
facts and the redacted field-level plan. Generated Admin TypeScript contracts in
`apps/admin-web` and `web` are synchronized with the Rust generator.

Web Admin now has a separate Metadata Authority apply page at
`/admin/automation/generated-artifacts/metadata-apply?artifact_id=...`.
Accepted review results can navigate there, but review acceptance still does
not mutate Canonical Metadata. The page loads the read-only apply-plan, shows
only redacted field summaries, disables final apply for fixture/fallback plans,
requires an explicit preparation step, and submits the final live mutation with
a stable UI idempotency key prefixed by
`web-generated-artifact-metadata-apply:`.

GAMA-060 browser evidence lives at:
`target/gama060-apply-plan-desktop.png`,
`target/gama060-apply-plan-mobile.png`,
`target/gama060-apply-result-desktop.png`, and
`target/gama060-apply-result-mobile.png`.

Closeout evidence and review findings are in `CLOSEOUT.md` and
`EVIDENCE_AND_GATES.md`.

## Active Task

- Task ID: none
- Lane: `library-metadata-control-plane`
- Status: closed
- Owner: none

Goal: complete. Open a focused follow-on before expanding this workflow.

## Key Files

- `crates/nako-core/src/automation.rs`
- `crates/nako-core/src/media/metadata.rs`
- `crates/nako-core/src/repository/metadata.rs`
- `crates/nako-api/src/admin/automation.rs`
- `crates/nako-db/src/sqlite/metadata.rs`
- `crates/nako-db/src/sqlite/automation.rs`
- `crates/nako-db/src/postgres/addons_automation.rs`
- `crates/nako-db/src/postgres/metadata_catalog.rs`
- `crates/nako-server/src/app/automation.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/app/metadata_application.rs`
- `crates/nako-server/src/app/tests/automation.rs`
- `apps/admin-web/src/adminApi/generated/contract.ts`
- `web/src/api/admin/generated/contract.ts`
- `web/src/api/admin/client.ts`
- `web/src/api/admin/read-models-data-source.ts`
- `web/src/api/admin/mutations-data-source.ts`
- `web/src/features/admin/admin-generated-artifact-review.tsx`
- `web/src/features/admin/admin-generated-artifact-metadata-apply.tsx`
- `web/src/features/admin/admin-surface.tsx`
- `web/src/shell/nako-router.tsx`
- `docs/workstreams/web-admin-generated-artifact-review-mutations/ROUTE_API_READINESS.md`
- `docs/workstreams/metadata-application-policy-seam/`
- `docs/workstreams/metadata-application-cross-path-audit/`

## Decisions

- Keep review acceptance and metadata apply as separate Admin operations.
- Apply-plan route is `POST /admin/v1/automation/generated-artifacts/{artifact_id}/metadata-apply-plan`.
- Apply-plan response is `AdminGeneratedArtifactMetadataApplyPlanResponse`.
- Reuse server-owned `MetadataApplication` for final apply; do not move it into
  `nako-metadata` without new cross-crate pressure.
- Start with one accepted `MetadataSuggestion` targeting a `MediaItem`.
- Treat raw Generated Artifact payload as privileged internal data.
- Generated Artifact metadata apply uses `MetadataSource::User` with
  `MetadataApplicationLockScope::ProtectAllLocks`; addon metadata writes retain
  source-relative lock protection.
- `commit_metadata_application` is a generic item plus catalog/search projection
  transaction. It is not durable apply audit persistence.
- `GeneratedArtifactMetadataApplyRequest` requires an explicit
  `idempotency_key`. Reusing the same key returns the persisted outcome as an
  idempotent replay.
- Durable apply outcomes live in
  `generated_artifact_metadata_apply_outcomes` with SQLite/PostgreSQL parity and
  status values `applied`, `noop`, and `failed`.
- Final Admin apply route is
  `POST /admin/v1/automation/generated-artifacts/{artifact_id}/metadata-apply`.
- Final Admin apply request body is
  `AdminGeneratedArtifactMetadataApplyRequest { idempotency_key }`.
- Final Admin apply response is `AdminGeneratedArtifactMetadataApplyResponse`
  and must stay redacted.
- Web Metadata Authority apply route is
  `/admin/automation/generated-artifacts/metadata-apply?artifact_id=...`.
- Web fixture and fallback data sources can render the plan, but must not claim
  to execute final apply.
- Low-frequency mock-only Admin settings pages were compacted to placeholders to
  keep the Admin route inside the existing bundle budget after adding the real
  GAMA workflow.

## Blockers

- None.

## Follow-Ons

- `proposed:generated-artifact-bulk-metadata-apply`: batch apply semantics,
  queueing, partial failure display, and per-item idempotency.
- `proposed:generated-artifact-provider-mapping-breadth`: provider-specific
  metadata patch mapping beyond the neutral first `MetadataSuggestion` shape.
- `proposed:generated-artifact-apply-operations-repair`: operator repair,
  replay diagnostics, and outcome audit search for failed/noop/applied results.
- `proposed:admin-settings-api-backed-restoration`: restore the placeholder
  Admin settings pages with real Admin API-backed controls while preserving
  bundle budgets.

## Watchpoints

- Do not make `/review` or `/metadata-apply-plan` apply metadata.
- The final `/metadata-apply` Admin route requires a stable idempotency key and
  returns only redacted apply outcome facts.
- Any follow-up Web work should consume the generated contract; do not hand-write a
  conflicting Web fixture contract.
- Do not expose raw `artifact_json`, prompt, source locators, paths, or secrets.
- Do not skip field lock and library refresh mode checks.
- Keep direct repository duplicate-key behavior as a storage error; route/app
  replay should fetch by `(artifact_id, idempotency_key)` before committing.
