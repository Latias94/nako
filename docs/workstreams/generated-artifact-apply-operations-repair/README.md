# Generated Artifact Apply Operations Repair

Status: Closed
Last updated: 2026-06-02

This workstream opens the next `library-metadata-control-plane` follow-on after
Generated Artifact one-artifact apply, bulk apply, and Provider Mapping breadth
all shipped.

The problem is no longer whether Nako can apply accepted Generated Artifacts.
The problem is whether operators can understand and safely recover from stale,
failed, skipped, or noop apply outcomes without raw internal access or blind
retries.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CONTEXT.jsonl`
- `HANDOFF.md`
- `CLOSEOUT.md`

Closed execution:

- `GAOR-010` opened the lane from GAMA/GABMA/GAPM closeout evidence.
- `GAOR-020` audited current durable outcome/batch records and chose the
  smallest repair-oriented Admin surface.
- `GAOR-030` shipped Admin outcome list/detail reads and a read-only recovery
  queue across core, DB, API, server, generated contracts, and Web Admin
  read-model mapping.
- `GAOR-040` closed the lane and split UI rendering plus bounded repair
  mutation into explicit follow-ons.

Boundary:

- keep Metadata Authority rules, target freshness checks, idempotency, and
  redaction intact;
- do not create a second hidden apply executor;
- do not broaden into provider-depth precision, generic job retry UI, or
  unbounded automation policy inside the first slice.
