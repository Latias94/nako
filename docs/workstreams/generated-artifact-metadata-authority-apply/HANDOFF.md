# Generated Artifact Metadata Authority Apply - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

`GAMA-010` opened the workstream and audited the boundary.

Generated Artifact review acceptance is already guarded: it can mark a metadata
suggestion as accepted, but it returns a boundary that says Canonical Metadata
was not changed and Metadata Authority apply is still required.

There is no Generated Artifact metadata apply-plan or apply route yet.

## Active Task

- Task ID: `GAMA-020`
- Status: ready
- Owner: unassigned

Goal: add a redaction-safe, read-only metadata apply-plan backend contract for
accepted metadata Generated Artifacts. This task must not mutate
`MediaItem.metadata`.

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
- Reuse server-owned `MetadataApplication` for final apply when mutation is
  introduced; do not move it into `nako-metadata` without new cross-crate
  pressure.
- Start with one accepted `MetadataSuggestion` targeting a `MediaItem`.
- Treat raw Generated Artifact payload as privileged internal data.

## Blockers

- None for `GAMA-020`.

## Watchpoints

- Do not make `/review` apply metadata.
- Do not expose raw `artifact_json`, prompt, source locators, paths, or secrets.
- Do not skip field lock and library refresh mode checks.
- If apply persistence is required, add SQLite/PostgreSQL parity before Web
  controls claim durable apply.
