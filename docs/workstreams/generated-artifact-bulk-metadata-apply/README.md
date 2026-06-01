# Generated Artifact Bulk Metadata Apply

Status: Active
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

Current task:

- `GABMA-020`: design and implement a read-only bulk apply-plan contract before
  any bulk mutation route exists.

Boundary:

- Review acceptance still does not mutate Canonical Metadata.
- Bulk apply must reuse the single-artifact apply plan and final apply
  authority semantics.
- Provider-specific mapping breadth, outcome repair tooling, and Admin
  settings restoration remain separate workstreams.
