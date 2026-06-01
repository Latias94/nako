# Generated Artifact Bulk Metadata Apply

Status: Closed
Last updated: 2026-06-01

This workstream turns the one-artifact Generated Artifact Metadata Authority
apply workflow into a guarded bulk apply workflow for accepted metadata
Generated Artifacts.

The existing GAMA lane proved a single-artifact path: read-only apply plan,
field-lock-aware mutation, durable idempotent outcome, and Web Admin
confirmation. This lane adds selection, batch planning, durable execution,
partial-failure reporting, and Web operator ergonomics without changing review
acceptance semantics.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CONTEXT.jsonl`
- `HANDOFF.md`
- `CLOSEOUT.md`

Closeout:

- `GABMA-070` closed this lane after fresh Rust/Web/PostgreSQL verification and
  closeout review.

Shipped behavior:

- Admin bulk plan:
  `POST /admin/v1/automation/generated-artifacts/metadata-apply-plan`.
- Admin bulk confirm:
  `POST /admin/v1/automation/generated-artifacts/metadata-apply-batches`.
- Admin batch status/result:
  `GET /admin/v1/automation/generated-artifacts/metadata-apply-batches/{batch_id}`.
- Confirmed batches persist a durable batch and job, execute through the
  existing one-artifact Metadata Authority apply path, and record per-item
  applied/noop/stale/failed/skipped outcomes.
- Web Admin supports accepted-artifact selection, redacted plan display,
  live-only confirmation, and redacted partial-result display.

Boundary:

- Review acceptance still does not mutate Canonical Metadata.
- Bulk apply must reuse the single-artifact apply plan and final apply
  authority semantics.
- Provider-specific mapping breadth, outcome repair tooling, and Admin
  settings restoration remain separate workstreams.
