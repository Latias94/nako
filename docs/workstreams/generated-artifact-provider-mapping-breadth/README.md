# Generated Artifact Provider Mapping Breadth

Status: Active
Last updated: 2026-06-01

This workstream extends accepted metadata Generated Artifacts beyond neutral
Canonical Metadata field patches so they can also propose Provider Subject and
Provider Mapping updates through the same guarded Metadata Authority workflow.

The predecessor lanes shipped safe review, one-artifact apply, and bulk apply:

- `generated-artifact-metadata-authority-apply` separated review acceptance
  from Canonical Metadata mutation and added idempotent single-artifact apply.
- `generated-artifact-bulk-metadata-apply` added bounded bulk planning,
  durable execution, partial-failure reporting, and Web Admin controls.

This lane keeps those authority boundaries and adds provider identity breadth:
an accepted metadata Generated Artifact may propose a host-owned Provider
Mapping for its target Media Item, but review acceptance still does not mutate
Canonical Metadata or provider mappings.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CONTEXT.jsonl`
- `HANDOFF.md`

Current execution:

- `GAPM-010` opened the lane and reconciled architecture routing.
- `GAPM-020` shipped redaction-safe read-only provider mapping plan support for
  the existing Generated Artifact metadata apply workflow.
- `GAPM-030` shipped durable/idempotent Provider Subject and accepted Provider
  Mapping apply through the single-artifact Metadata Authority outcome
  transaction.
- Next task: `GAPM-040`, bulk/Admin counter and outcome reconciliation.

Boundary:

- Generated Artifact review acceptance remains a staging action.
- Final apply remains Admin-only, idempotent, target-freshness checked, and
  redacted.
- Provider Mapping persistence must use Nako-owned Provider Subject and
  Provider Mapping repositories, not raw addon/provider payloads.
- Broader provider depth, provider search, apply repair tooling, Admin settings
  restoration, and Public Client API changes stay outside this lane.
