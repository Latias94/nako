# Durable Job Queue And Resource Classes - Milestones

Status: Closed
Last updated: 2026-05-29

## M0 - Scope And Evidence Freeze

Exit criteria:

- The lane is opened from the Control Plane proposed candidate.
- Non-goals prevent crate churn, external queue adoption, and broad worker
  rewrites in the first slice.
- Architecture indexes point to the real workstream directory.

Primary evidence:

- `docs/workstreams/durable-job-queue-and-resource-classes/DESIGN.md`
- `docs/workstreams/durable-job-queue-and-resource-classes/TODO.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/architecture/WORKSTREAM_LINKS.md`

## M1 - Process-Local Resource Class Registry

Exit criteria:

- Runtime resource classes have one registry.
- Existing scan, metadata, and webhook permits are registry-owned.
- Diagnostics expose name, max permits, and available permits in stable order.
- Duplicate class names are rejected.

Primary gates:

```powershell
cargo nextest run -p nako-server runtime_resource_class --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

## M2 - Durable Job Class To Budget Mapping

Exit criteria:

- Durable job classes map to budget classes explicitly.
- Unknown classes have a deliberate policy.
- The mapping does not rely on string prefix inference.

Primary gates:

```powershell
cargo nextest run -p nako-server job_resource_class --no-fail-fast
cargo check -p nako-server --tests
```

## M3 - Scheduler Admission Tracer Bullet

Exit criteria:

- One durable job family is admitted through scheduler policy.
- Leases, cancellation checkpoints, and typed executor ownership are preserved.
- Budget saturation leaves work queued rather than spawning unbounded tasks.

Primary gates:

```powershell
cargo nextest run -p nako-server job_scheduler --no-fail-fast
cargo nextest run -p nako-server job_runtime --no-fail-fast
```

## M4 - Retry, Backoff, And Queue Pressure

Exit criteria:

- Retry/backoff is explicit and persisted where needed.
- Cancellation remains distinct from retryable failure.
- Queue pressure diagnostics are redacted and grouped by safe classes.

Primary gates:

```powershell
cargo nextest run -p nako-db job_retry --no-fail-fast
cargo nextest run -p nako-server queue_pressure --no-fail-fast
cargo check -p nako-core -p nako-db -p nako-api -p nako-server --tests
```

## M5 - Closeout

Exit criteria:

- Fresh gate evidence is recorded.
- Residual work is completed or split by boundary.
- `WORKSTREAM.json` status matches reality.
- `HANDOFF.md` names the next highest-leverage follow-on.

Result:

- Closed on 2026-05-29 after focused resource registry, mapping, scheduler,
  runtime, retry, queue pressure, formatting, JSON, and diff gates passed.
- Split priority policy and broader scheduler migration to
  `proposed:durable-job-priority-policy-and-scheduler-migration`.
