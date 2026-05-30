# Generated Artifact Metadata Authority Apply - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

`GAMA-020` is complete.

Generated Artifact review acceptance is still guarded, and the new read-only
metadata apply-plan route now exposes field-level, redacted plan facts without
mutating Canonical Metadata.

## Active Task

- Task ID: `GAMA-030`
- Status: ready
- Owner: unassigned

Goal: add host-owned apply execution for executable apply plans, preserving
field locks and revalidating target freshness before mutation.

## Key Files

- `crates/nako-core/src/automation.rs`
- `crates/nako-api/src/admin/automation.rs`
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
- Reuse server-owned `MetadataApplication` for final apply when mutation is
  introduced; do not move it into `nako-metadata` without new cross-crate
  pressure.
- Start with one accepted `MetadataSuggestion` targeting a `MediaItem`.
- Treat raw Generated Artifact payload as privileged internal data.

## Blockers

- None for `GAMA-030`.

## Watchpoints

- Do not make `/review` or `/metadata-apply-plan` apply metadata.
- Do not expose raw `artifact_json`, prompt, source locators, paths, or secrets.
- Do not skip field lock and library refresh mode checks.
- If apply persistence is required, add SQLite/PostgreSQL parity before Web
  controls claim durable apply.
