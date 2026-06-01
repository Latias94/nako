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

Bulk apply is not implemented yet. This lane starts with a read-only bulk
apply-plan contract before adding durable batch mutation.

## Active Task

- Task ID: `GABMA-020`
- Lane: `library-metadata-control-plane`
- Status: active
- Owner: codex

Goal: add a redaction-safe, read-only bulk apply-plan contract for selected
accepted metadata Generated Artifacts.

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

- None for `GABMA-020`.

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
