# 0006: Persist Job Inputs and Use Explicit Retry Policy

## Status

Proposed

## Context

Taru background jobs will eventually cover library scans, metadata refresh,
NFO import/export, webhook delivery, automation, remote-storage cache work, and
transcode sessions. Those jobs need to be inspectable and recoverable after a
server restart.

Persisting only job status is not enough. A failed job needs enough original
input to explain what ran, support a future retry action, and let operators
understand side effects. At the same time, retries and cancellation cannot be
generic fire-and-forget behavior because different resource classes have
different safety profiles.

## Decision

Every persisted job stores an `input_json` payload. The payload is the durable
request envelope for that job kind. It must contain stable IDs and explicit
options, not process-local handles or paths that bypass Taru's VFS model.

Job lifecycle states:

- `queued`: accepted and waiting for a runner
- `running`: currently owned by a runner lease
- `succeeded`: completed successfully with optional `summary_json`
- `failed`: completed unsuccessfully with a safe error message
- `cancelled`: terminal cancellation acknowledged by the owning runner or by a
  queued-job cancellation path with no side effects in flight

Ownership and leases:

- running jobs must be fenced by a run token, not only by job ID
- worker identity is diagnostic; the run token is the authority for heartbeat,
  completion, failure, and cancellation acknowledgement
- leased jobs record heartbeat and lease expiry timestamps
- stale running jobs are recovered by an explicit lease-aware policy
- queued jobs must not be failed by startup recovery merely because no process
  was alive at restart

Retry policy:

- retries are explicit per job kind and resource class
- retries create a new job row that copies the previous `input_json`
- retry attempts must not mutate canonical metadata without idempotency checks
- provider, webhook, remote storage, and automation retries need per-provider
  limits and backoff before implementation
- transcode retries must not reuse stale process state or temporary-session IDs

Cancellation policy:

- cancellation is a requested state transition, not an unsafe thread kill
- cancellable jobs must define a checkpoint or orchestration boundary
- external processes such as FFmpeg must be stopped by their owning runner
- cancellation request and cancellation acknowledgement are distinct; running
  jobs become terminal `cancelled` only after the current run token observes
  and persists acknowledgement
- queued jobs may become terminal `cancelled` immediately when no runner owns
  them

Idempotency and failure isolation:

- repeated library scans are expected and must remain idempotent
- batch jobs should isolate per-item failures by default
- strict all-or-nothing behavior must be an explicit job option
- summaries should include counts and failure lists where useful

Resource budgets:

- every runner must declare a resource class
- resource classes are bounded independently
- initial classes include `disk.scan` and `media.probe`
- future classes include `network.metadata`, `network.webhook`,
  `automation.external_api`, `storage.remote`, `cpu.transcode`, and
  `gpu.transcode`

## Consequences

- Jobs are more auditable and easier to recover.
- Future retry APIs can be implemented without guessing original user intent.
- Some payload schemas will need migration or compatibility handling over time.
- Sensitive fields must not be stored directly in `input_json`; store secret
  references instead.

## Alternatives Considered

- In-memory job inputs only: simpler, but not restart-safe and not auditable.
- Mutating failed jobs back to queued for retries: compact, but loses attempt
  history and makes debugging harder.
- One global retry policy: easy to implement, but unsafe across metadata,
  webhook, automation, storage, and transcode workloads.

## Related Workstreams

- `docs/workstreams/server-foundation/`
