# Addon Task Runtime Contract

Status: Completed
Last updated: 2026-05-23

## Why This Lane Exists

Nako already knew how to validate Addon manifests, record Addon Task
declarations, and turn them into explicit routing plans. This lane added the
host-owned lifecycle around task execution, progress, results, cancellation,
retry, direct sidecar task-path dispatch, and audit.

## Problem

Addon Task declarations exist, but the runtime boundary is still easy to blur.
If task names or examples become the contract too early, Nako can accidentally
turn declaration samples into a product promise before host-owned execution
semantics are stable.

## Target State

When this lane closes, Nako should be able to:

- represent an Addon Task run as a Nako-owned lifecycle object separate from
  the declaration itself;
- report task progress, result, failure, and cancellation through redaction-
  safe host-owned surfaces;
- keep retry/backoff and idempotency explicit rather than hidden in addons;
- preserve Addon Task declaration routing without forcing the addon side to own
  host scheduling policy;
- keep source catalog, marketplace, package signing, and process supervision as
  separate lanes.

## Scope

- Addon Task run model and state transitions.
- Host-owned progress/result/cancellation/retry semantics.
- Admin-facing task diagnostics and audit.
- Integration with existing job/outbox/durable-job boundaries.
- Validation and docs for the first runtime task slice.

## Non-Goals

- Addon source catalog / marketplace discovery.
- Package signing trust roots.
- Direct container or process supervision.
- Broad provider breadth.
- Native Plugin ABI or in-process addon execution.
- Changing the manifest declaration language beyond what is needed for runtime
  evidence.

## Architecture Direction

Treat Addon Task as a declaration plus a host-owned run. The addon declares what
it can do; Nako owns the execution record, state transitions, progress
surface, final result, retry policy, and audit trail.

The contract should clearly separate:

- declaration time: what the addon says it supports;
- scheduling time: when Nako decides a run may start;
- run time: progress/result/cancellation/state updates;
- completion time: outcome persistence and downstream effects.

`bulk-metadata-scrape` should remain an example declaration until this lane
defines the runtime contract. It should not be treated as a promise that an
addon can own progress/result semantics without the host boundary.

## Implemented First Slice

The first runtime slice represents an Addon Task run as:

- a `JobKind::AddonTask` job for host-owned scheduling, status, cancellation,
  retry visibility, and Admin job integration;
- an `addon_task_runs` runtime record keyed by `job_id` for addon declaration
  snapshot, idempotency key, attempt number, retry lineage, progress, result,
  and safe error code;
- stable JSON schemas:
  - `nako.addon.task_run.input.v1`
  - `nako.addon.task_run.progress.v1`
  - `nako.addon.task_run.result.v1`

The run is created by Nako through Admin API after manifest declaration,
grants, and routing-plan executability are validated. Syncing routing plans
continues to create no hidden work.

The Addon Sidecar can only claim and update a run through Addon Token protected
runtime endpoints. Claiming a run issues a `run_token`; progress, completion,
failure, and cancellation acknowledgement must present that fenced guard. This
keeps stale sidecar workers from overwriting a newer or already terminal run.
The claim/progress lease payload also carries the host-authored execution input
for the run, so the sidecar can execute the task without the admin-facing
summary exposing that payload.

Cancellation remains host-owned. Operators request cancellation through the
existing Admin job cancellation route. The Addon Sidecar observes
`cancel_requested_at` on progress/heartbeat-style updates and acknowledges
cancellation through the task runtime endpoint.

Retry is also host-owned. A failed Addon Task run can be retried through Admin
API only while `attempt < max_attempts`. The retried run receives a new job id,
new idempotency key, incremented attempt, and `retry_of_job_id` pointing to the
failed run.

The runtime now supports two dispatch modes:

- `sidecar_claim` keeps the first worker contract: an Addon Sidecar claims a
  queued run and reports progress/result through fenced runtime endpoints.
- `direct` asks Nako to claim the specific newly created or retried run and
  call the declared sidecar task `path` directly with a host-owned task
  envelope.

Direct dispatch is layered on top of the same run model. It does not make
manifest declaration a scheduling primitive, does not let Addon Source Catalog
or marketplace discovery create hidden work, and does not move retry policy
into the sidecar. A direct run sends exactly one HTTP task-path dispatch per
host-owned run; Admin retry creates the next host-owned run when another
attempt is allowed.

Direct dispatch completion still flows through Nako-owned terminal updates. On
success Nako records progress and result. On HTTP/protocol failure Nako records
a safe failure code. If host cancellation is requested while the sidecar call is
in flight, the post-call heartbeat observes `cancel_requested_at` and Nako
records the run as cancelled instead of succeeded.

The current direct path uses the existing outbound Addon client and therefore
requires an outbound sidecar authentication secret when the manifest declares
`Bearer` or `SharedSecret` auth. This lane does not introduce secret storage or
credential grant onboarding; direct dispatch tests and the shipped first slice
cover `AddonAuth::None`, while authenticated outbound task dispatch remains a
credential-management follow-on.

## Reference Patterns

- Jellyfin shows task-like plugin features, but its in-process model is not the
  Nako target; Nako needs a host-owned runtime object instead of a plugin ABI.
- Home Assistant and VS Code both separate manifest intent from runtime
  execution policy; Nako should do the same for task declarations.
- Obsidian-style version gating is a reminder that compatibility metadata
  should be explicit and separate from feature names.

## Related Docs

- `docs/workstreams/addon-runtime-and-distribution/`
- `docs/workstreams/addon-manager-lifecycle-automation/`
- `docs/workstreams/addon-source-catalog-marketplace/`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/adr/0033-version-addon-protocol-independently-from-addon-and-crate-releases.md`

## Closeout

This lane closed in ATRC-060 after ATRC-040 shipped direct dispatch. The
runtime contract covers host-owned Addon Task runs, sidecar-claimed execution,
direct task-path dispatch, progress/result persistence, retry, cancellation,
and safe failure classification.

Follow-ons:

- authenticated outbound sidecar task dispatch credential management for
  `AddonAuth::Bearer` and `AddonAuth::SharedSecret`;
- official-addon task-path smoke coverage once an official addon exposes a task
  declaration;
- Addon Source Catalog / marketplace discovery;
- package signing and trust-root policy;
- provider breadth beyond the first companion addon;
- process/container supervision.
