# Generated Artifact Metadata Authority Apply - Milestones

Status: Active
Last updated: 2026-05-30

## M0 - Lane Opened

Exit criteria:

- Existing Generated Artifact review, Admin API, MetadataApplication, and NFO
  apply boundaries are audited.
- Workstream docs and architecture links exist.
- First execution task is narrow enough for one agent.

Status: Complete via `GAMA-010`.

## M1 - Read-Only Apply Plan

Exit criteria:

- Accepted metadata Generated Artifacts have a redacted apply-plan contract.
- Plan generation rejects unsupported kinds, stale targets, non-accepted
  artifacts, and invalid payload shapes without mutation.
- Tests prove `MediaItem.metadata` is unchanged after plan generation.

Status: Complete via `GAMA-020`.

## M2 - Host-Owned Apply

Exit criteria:

- Apply revalidates the current target and delegates final mutation through
  Nako-owned metadata application policy.
- User/source locks and library refresh mode are honored.
- Catalog/search projection updates with the metadata mutation.
- Raw generated payload and private target facts stay out of Admin responses.

Status: Complete via `GAMA-030`.

## M3 - Idempotency, Persistence, And Admin API

Exit criteria:

- Apply replay behavior is explicit and tested.
- Any new persistence has SQLite/PostgreSQL parity and migration evidence.
- Final Admin routes and generated contracts are stable enough for Web.

Status: In progress. `GAMA-040` completed durable outcome persistence,
idempotency-key replay, and SQLite/PostgreSQL contract evidence. `GAMA-050`
still needs the final Admin route and generated wire contracts.

## M4 - Web Admin Workflow

Exit criteria:

- Web Admin shows apply plan, blocked reasons, skipped locked fields, and final
  outcome.
- Review acceptance and Metadata Authority apply remain separate user actions.
- Browser smoke covers desktop and mobile without layout overlap or fixture-only
  success.

## M5 - Closeout

Exit criteria:

- Backend, Web, JSON, formatting, diff, and browser gates are recorded.
- `WORKSTREAM.json` evidence is current.
- Bulk apply, provider-specific enrichment, or operations repair follow-ons are
  split instead of being hidden in this lane.
