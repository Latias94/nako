# Addon Event Scheduler And Replay — Milestones

Status: Active
Last updated: 2026-05-25

## M0 — Scope And Evidence Freeze

Exit criteria:

- Problem, target state, non-goals, and authority are explicit.
- Addon Ecosystem Foundation is referenced as the predecessor lane.
- First implementation slice is due work selection, not a background loop.

Primary evidence:

- `docs/workstreams/addon-event-scheduler-and-replay/DESIGN.md`
- `docs/workstreams/addon-event-scheduler-and-replay/TODO.md`

## M1 — Due Work Selection

Status: Complete.

Exit criteria:

- Scheduler candidates can be listed from durable state.
- Candidates account for enabled addons, executable event routing plans, grants,
  already succeeded attempts, retry timestamps, and attempt limits.
- Admin diagnostics remain redaction-safe.

Primary gates:

- `cargo nextest run -p nako-db addon_event_scheduler --no-fail-fast`
- focused server scheduler diagnostics tests

## M2 — In-Flight Guard And Automatic Retry

Status: Complete.

Exit criteria:

- Concurrent scheduler workers cannot duplicate the same tuple.
- Retryable failures become due only after `next_retry_at`.
- Exhausted attempts are visible and not retried automatically.

Primary gate:

- `cargo nextest run -p nako-server addon_event_scheduler --no-fail-fast`

## M3 — Scheduler Runtime Integration

Exit criteria:

- Scheduler lifecycle starts and stops with the server runtime.
- Test harness controls timing deterministically.
- Manual event delivery remains available for operator diagnostics.

Primary gate:

- `cargo nextest run -p nako-server addon_event_scheduler --no-fail-fast`

## M4 — Forced Replay And Filters

Exit criteria:

- Forced replay is a separate admin action with explicit operator intent.
- Normal scheduling continues to skip already succeeded subscriptions.
- Event subscription filters execute before sidecar calls or are split by ADR.

Primary gates:

- `cargo nextest run -p nako-server addon_event_replay --no-fail-fast`
- `cargo nextest run -p nako-server addon_event_filter --no-fail-fast`

## M5 — Closeout

Exit criteria:

- Evidence is recorded.
- `WORKSTREAM.json` status reflects final state.
- Notification bridge and other breadth work are either opened or deferred.
