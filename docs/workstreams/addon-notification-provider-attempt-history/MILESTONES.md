# Addon Notification Provider Attempt History — Milestones

Status: Complete
Last updated: 2026-05-25

## M0 — History Contract

Exit criteria:

- Retention size and safe fields are explicit.
- Persistent history and host schema changes are out of scope.

## M1 — Bounded Recorder

Exit criteria:

- Recorder stores only safe fields.
- Capacity is bounded and tested.

## M2 — Provider Send Integration

Exit criteria:

- Success, disabled, retryable failure, and non-retryable failure are recorded.
- Existing host retry behavior is unchanged.

## M3 — Diagnostics And Docs

Exit criteria:

- Diagnostics expose recent attempts safely.
- Operator docs explain limits and non-persistence.

## M4 — Closeout

Exit criteria:

- Final gates pass and persistent history is deferred or split.

Result: complete. Final gates passed; persistent sidecar history and Admin UI
history are deferred.
