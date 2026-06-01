# Generated Artifact Apply Operations Repair

Status: Active
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

Current execution:

- `GAOR-010` opened the lane from GAMA/GABMA/GAPM closeout evidence.
- `GAOR-020` is the first executable task: audit current durable outcome/batch
  records and choose the smallest repair-oriented Admin surface.

Boundary:

- keep Metadata Authority rules, target freshness checks, idempotency, and
  redaction intact;
- do not create a second hidden apply executor;
- do not broaden into provider-depth precision, generic job retry UI, or
  unbounded automation policy inside the first slice.
