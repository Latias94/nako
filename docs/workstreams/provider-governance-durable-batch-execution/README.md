# Provider Governance Durable Batch Execution

Status: Closed
Last updated: 2026-06-02

This workstream turns bounded synchronous Metadata Candidate Review batch apply
into a durable, job-backed Admin control-plane workflow.

It follows `provider-governance-bulk-review`, which shipped selected-review
batch planning, bounded synchronous confirmation, and Web Admin governance. The
durable execution problem was not another Provider Mapping executor; it was a
persisted batch and job envelope for larger, retryable, cancellable operator
work.

## Final State

- Closed after `PGDBE-060`.
- Durable batch state, Admin create/status, job-backed execution, and Web Admin
  durable status are shipped.
- Remaining scope is split to focused follow-ons for related hierarchy
  application, Public Client API exposure, provider endpoint breadth,
  scheduler priority policy, and audit/undo governance.

## Authoritative Docs

- `DESIGN.md`: scope, target state, architecture direction, and source
  coverage.
- `TODO.md`: human task ledger.
- `TASKS.jsonl`: machine-readable task state.
- `CAMPAIGNS.jsonl`: approved or draft autonomous task bundles.
- `EVIDENCE_AND_GATES.md`: validation commands and evidence.
- `WORKSTREAM.json`: machine-readable workstream summary.
- `HANDOFF.md`: continuation state.
- `CLOSEOUT.md`: final shipped scope, gates, follow-ons, and residual risks.

## Boundaries

- Admin-only.
- Existing single-review `MetadataCandidateReviewApplicationService` remains
  the apply authority.
- Durable batch state is Candidate Review specific and must not reuse Generated
  Artifact apply outcome tables.
- Execution must use durable job/runtime boundaries from ADR 0053, not hidden
  raw background tasks.
- Related hierarchy application, Public Client API exposure, provider endpoint
  breadth, and audit/undo governance remain separate follow-ons.
