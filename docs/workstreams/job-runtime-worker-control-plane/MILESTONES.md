# Job Runtime Worker Control Plane Milestones

Status: Completed
Last updated: 2026-05-19

## M0 - Runtime Inventory And Contract

Status: Done

Exit criteria:

- Existing job execution surfaces are listed.
- ADR 0006 retry/cancel guidance and ADR 0019 runtime-supervisor guidance are
  reconciled with current code.
- First shared worker contract is specific enough to implement.

## M1 - Managed Artwork Worker Tracer Bullet

Status: Done

Exit criteria:

- A supervised runtime worker can process queued Managed Artwork ingest work.
- The worker uses the same safe fetch/validate/store/fail pipeline as the
  manual Admin `process-next` route.
- Resource limits are explicit and process-local.
- Public Client API shape is unchanged.

Progress:

- `JRWCP-020` adds an opt-in process-local worker for Managed Artwork ingest.
- `JRWCP-030` adds typed startup recovery for claimed Managed Artwork ingests
  and preserves queued artwork work across restart.

## M2 - Recovery And Cancellation Semantics

Status: Split

Exit criteria:

- Failed worker execution produces safe summaries.
- Stale running jobs have an explicit recovery policy.
- Cancellation is implemented only if a runner can observe it, or split as a
  separate follow-on with the missing state-machine decisions named.

Progress:

- Recovery policy is explicit for Managed Artwork ingest.
- Cancellation is split into a later durable job ownership/control-plane lane.

## M3 - Closeout

Status: Done

Exit criteria:

- Required gates pass with fresh evidence.
- Remaining job kinds are listed as follow-ons.
- Workstream docs and HTTP/Admin docs are consistent with shipped behavior.

Result:

- Lane closes after `JRWCP-020` and `JRWCP-030`.
- `JRWCP-040` cancellation is deliberately deferred because truthful
  cancellation needs durable ownership/lease or a typed cancellation state.
