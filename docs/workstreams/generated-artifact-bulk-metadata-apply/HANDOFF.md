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

`GABMA-020` added the read-only bulk apply-plan contract and route. `GABMA-030`
added durable batch request persistence. `GABMA-040` added durable job-backed
batch execution through the existing one-artifact apply path with per-item
outcomes and partial-failure accounting. `GABMA-050` exposed the Admin
confirm/status/result HTTP surface and synchronized generated Admin TypeScript
contracts. The next task is the Web Admin workflow.

## Active Task

- Task ID: `GABMA-060`
- Lane: `library-metadata-control-plane`
- Status: active
- Owner: codex

Goal: add Web Admin bulk metadata apply planning, confirmation, live-only
mutation, and partial-result display.

## Completed Evidence

- `GABMA-020`: `POST /admin/v1/automation/generated-artifacts/metadata-apply-plan`
  now accepts `{ "artifact_ids": [...] }` and returns a redacted plan with
  selection counters, per-artifact planned/missing items, aggregate status and
  field-action counters, and no Canonical Metadata mutation.
- Validated with focused API/server gates, Admin generated contract sync,
  format check, and `git diff --check`.
- `GABMA-030`: repository persistence now stores confirmed bulk batches with
  idempotent replay by batch idempotency key, selection/summary snapshots,
  per-item redacted plan snapshots and deterministic item idempotency keys,
  guarded batch status transitions, and rollback on failed item persistence.
- Validated with focused `nako-db` and `nako-server` bulk metadata apply gates,
  format check, and `git diff --check`.
- `GABMA-040`: confirmed batches now enqueue a durable
  `generated_artifact_metadata_bulk_apply` job and can be executed through
  `AutomationAppService::execute_generated_artifact_metadata_bulk_apply_batch`.
  Execution reuses the one-artifact Metadata Authority apply path, persists
  per-item applied/noop/stale/failed outcomes and derived execution counters,
  succeeds the durable job with a redacted summary, and replays terminal batch
  state without duplicate mutation.
- Validated with focused `nako-db` and `nako-server` bulk metadata apply gates.
- `GABMA-050`: Admin routes now expose batch confirmation and status/result
  read models:
  `POST /admin/v1/automation/generated-artifacts/metadata-apply-batches` and
  `GET /admin/v1/automation/generated-artifacts/metadata-apply-batches/{batch_id}`.
  Route tests cover auth, request body, idempotent replay, queued status,
  completed partial results, error mapping, and redaction. Generated Admin
  TypeScript contracts are synchronized for both `apps/admin-web` and `web`.
- Validated with focused `nako-api` contract/DTO gates, `nako-server` bulk
  metadata apply HTTP/app gates, contract generation, and format check.

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

- None for `GABMA-050`.

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
