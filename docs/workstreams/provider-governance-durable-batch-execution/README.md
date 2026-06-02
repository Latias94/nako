# Provider Governance Durable Batch Execution

Status: Active
Last updated: 2026-06-02

This workstream turns bounded synchronous Metadata Candidate Review batch apply
into a durable, job-backed Admin control-plane workflow.

It follows `provider-governance-bulk-review`, which shipped selected-review
batch planning, bounded synchronous confirmation, and Web Admin governance. The
next problem is not another Provider Mapping executor; it is a persisted batch
and job envelope for larger, retryable, cancellable operator work.

## Current Task

- `PGDBE-020`: add the core and database durable batch model, repository
  contract, job kind, and resource-class mapping.

## Authoritative Docs

- `DESIGN.md`: scope, target state, architecture direction, and source
  coverage.
- `TODO.md`: human task ledger.
- `TASKS.jsonl`: machine-readable task state.
- `CAMPAIGNS.jsonl`: approved or draft autonomous task bundles.
- `EVIDENCE_AND_GATES.md`: validation commands and evidence.
- `WORKSTREAM.json`: machine-readable workstream summary.
- `HANDOFF.md`: continuation state.

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
