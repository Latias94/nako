# Generated Artifact Bulk Metadata Apply - Handoff

Status: Active
Last updated: 2026-06-01

## Current State

This lane is open as the first implementation follow-on after
`generated-artifact-metadata-authority-apply`.

GAMA shipped the one-artifact Metadata Authority apply workflow:

- read-only apply-plan route;
- host-owned field-lock-aware final apply;
- durable apply outcomes with explicit idempotency keys;
- synchronized generated Admin TypeScript contracts;
- Web Admin confirmation route.

`GABMA-020` added the read-only bulk apply-plan contract and route. Bulk
mutation is still not implemented; the next task adds durable batch request
persistence before execution.

## Active Task

- Task ID: `GABMA-030`
- Lane: `library-metadata-control-plane`
- Status: active
- Owner: codex

Goal: persist a confirmed bulk apply request with batch identity, selection
snapshot, aggregate plan snapshot, state, and per-item idempotency seeds.

## Completed Evidence

- `GABMA-020`: `POST /admin/v1/automation/generated-artifacts/metadata-apply-plan`
  now accepts `{ "artifact_ids": [...] }` and returns a redacted plan with
  selection counters, per-artifact planned/missing items, aggregate status and
  field-action counters, and no Canonical Metadata mutation.
- Validated with focused API/server gates, Admin generated contract sync,
  format check, and `git diff --check`.

## Key Context

- `docs/workstreams/generated-artifact-metadata-authority-apply/`
- `docs/workstreams/web-admin-generated-artifact-review-mutations/`
- `docs/workstreams/metadata-application-policy-seam/`
- `docs/workstreams/metadata-application-cross-path-audit/`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/architecture/LANES.md`
- `docs/architecture/WORKSTREAM_LINKS.md`

## Decisions

- Start with bulk plan, not mutation.
- Bulk confirm should enqueue durable work instead of applying an unbounded
  batch inside the HTTP request path.
- Each item must apply through the existing one-artifact Metadata Authority
  apply path.
- Per-item idempotency must be deterministic and scoped by batch identity plus
  artifact identity.
- Partial failure is expected product behavior and must be visible.
- Fixture/fallback Web mode may render plans but must not claim live mutation.

## Blockers

- None for `GABMA-030`.

## Watchpoints

- Do not make review acceptance apply metadata.
- Do not bypass `MetadataApplication`, field locks, stale-target checks, or
  catalog/search projection commits.
- Do not expose raw artifact JSON, prompt, provider payload, Source Locator,
  path, token, or secret values.
- Do not mix provider-specific mapping breadth into this lane.
- Do not raise Web bundle budgets to hide low-frequency UI growth.

## Follow-Ons Outside This Lane

- `proposed:generated-artifact-provider-mapping-breadth`
- `proposed:generated-artifact-apply-operations-repair`
- `proposed:admin-settings-api-backed-restoration`
