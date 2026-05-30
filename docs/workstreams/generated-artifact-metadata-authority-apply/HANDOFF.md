# Generated Artifact Metadata Authority Apply - Handoff

Status: Active
Last updated: 2026-05-30

## Current State

`GAMA-030` is complete.

Generated Artifact review acceptance is still guarded, and the new read-only
metadata apply-plan route now exposes field-level, redacted plan facts without
mutating Canonical Metadata.

Accepted metadata Generated Artifacts now have a host-owned app-layer apply
execution path. The path rebuilds an executable plan, rejects stale targets
before mutation, applies through `MetadataApplication` with all field locks
protected, and commits Canonical Metadata plus catalog/search projection in one
metadata application transaction.

## Active Task

- Task ID: `GAMA-040`
- Status: ready
- Owner: unassigned

Goal: decide and implement durable apply audit/outcome persistence if request
local replay/no-op semantics are not enough for retries, repair, or operations
visibility before the final Admin apply route is exposed.

## Key Files

- `crates/nako-core/src/automation.rs`
- `crates/nako-core/src/media/metadata.rs`
- `crates/nako-core/src/repository/metadata.rs`
- `crates/nako-api/src/admin/automation.rs`
- `crates/nako-db/src/sqlite/metadata.rs`
- `crates/nako-db/src/postgres/metadata_catalog.rs`
- `crates/nako-server/src/app/automation.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/app/metadata_application.rs`
- `crates/nako-server/src/app/tests/automation.rs`
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

## Blockers

- None for `GAMA-030`.

## Watchpoints

- Do not make `/review` or `/metadata-apply-plan` apply metadata.
- Do not expose the final `/metadata-apply` Admin route until `GAMA-040` has
  either shipped durable audit persistence or recorded why request-local replay
  is sufficient.
- Do not expose raw `artifact_json`, prompt, source locators, paths, or secrets.
- Do not skip field lock and library refresh mode checks.
- If apply audit persistence is required, add SQLite/PostgreSQL parity before
  Web controls claim durable apply.
