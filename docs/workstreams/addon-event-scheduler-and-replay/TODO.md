# Addon Event Scheduler And Replay — TODO

Status: Complete
Last updated: 2026-05-25

## M0 — Scope And Evidence Freeze

- [x] AESR-010 [owner=planner] [deps=none] [scope=docs/workstreams/addon-event-scheduler-and-replay]
  Goal: Freeze scheduler/replay problem, target state, non-goals, and evidence
  gates after Addon Ecosystem Foundation closeout.
  Validation: `DESIGN.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`,
  `WORKSTREAM.json`, and `HANDOFF.md` exist and agree.
  Evidence: `docs/workstreams/addon-event-scheduler-and-replay/DESIGN.md`.
  Handoff: Start implementation at AESR-020.

## M1 — Due Work Selection

- [x] AESR-020 [owner=codex] [deps=AESR-010] [scope=crates/nako-core,crates/nako-db,crates/nako-server]
  Goal: Add a redaction-safe scheduler query that lists due addon event
  delivery work without loading or returning event payload values to admin
  diagnostics.
  Validation: `cargo nextest run -p nako-db addon_event_scheduler --no-fail-fast`
  and focused server diagnostics tests.
  Review: Check event status filtering, subscription matching, grant checks,
  and SQLite/PostgreSQL parity.
  Evidence: repository contract tests and scheduler candidate tests.
  Handoff: Due work selection is proven. AESR-030 can add durable in-flight
  guards and automatic retry without starting a background loop first.

## M2 — In-Flight Guard And Automatic Retry

- [x] AESR-030 [owner=codex] [deps=AESR-020] [scope=crates/nako-core,crates/nako-db,crates/nako-server]
  Goal: Prevent duplicate concurrent delivery for the same
  addon/event/subscription tuple and consume `next_retry_at` for automatic
  retries.
  Validation: `cargo nextest run -p nako-server addon_event_scheduler --no-fail-fast`.
  Review: Check crash recovery, lease expiry, max attempts, and resource budget
  behavior.
  Evidence: repository claim contract plus runtime tests proving one sidecar call
  under concurrent scheduler pressure, waiting retry skip, and retry after
  `next_retry_at` becomes due.
  Handoff: Durable claim now writes `running` attempts with `lease_expires_at`;
  AESR-040 can build a scheduler loop on top of the same claim API.

## M3 — Scheduler Runtime Integration

- [x] AESR-040 [owner=codex] [deps=AESR-030] [scope=crates/nako-server]
  Goal: Wire the scheduler loop into server runtime lifecycle with explicit
  configuration and bounded concurrency.
  Validation: `cargo nextest run -p nako-server addon_event_scheduler --no-fail-fast`.
  Review: Check shutdown, jitter/backoff, observability, and that manual admin
  delivery still works.
  Evidence: server lifecycle test plus HTTP scheduler loop tests for due-event
  dispatch and configured event concurrency.
  Handoff: Scheduler is disabled by default, starts only when configured, is
  supervised by the runtime, and reuses durable delivery claims. Continue at
  AESR-050 for forced replay and filters.

## M4 — Forced Replay And Filters

- [x] AESR-050 [owner=codex] [deps=AESR-040] [scope=crates/nako-api,crates/nako-server,crates/nako-db]
  Goal: Add explicit forced replay with operator intent and evaluate persisted
  event subscription filters before scheduling sidecar calls.
  Validation: `cargo nextest run -p nako-server addon_event_replay addon_event_filter --no-fail-fast`.
  Review: Confirm replay is separate from normal delivery and filter evaluation
  cannot expose payload values in responses.
  Evidence: HTTP tests prove forced replay creates an audited replay attempt
  after a successful normal delivery, matching event fact filters allow delivery,
  and non-matching filters skip scheduler/manual sidecar calls without echoing
  payload values.
  Handoff: Forced replay and simple JSON event fact filters are implemented.
  Split filter language expansion into an ADR if payload or nested predicates
  become necessary.

## M5 — Closeout

- [x] AESR-060 [owner=planner] [deps=AESR-050] [scope=docs/workstreams/addon-event-scheduler-and-replay]
  Goal: Close the scheduler/replay lane or split notification bridge as the
  next workstream.
  Validation: `cargo fmt --all -- --check`, focused nextest gates,
  `git diff --check`, and `WORKSTREAM.json` parse.
  Review: Run review-workstream and verify-rust-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`.
  Handoff: DONE. Scheduler/replay is operational and this lane is closed.
  Notification bridge is the next named follow-on and must be opened as a
  separate lane before implementation.
