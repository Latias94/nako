# Addon Notification Provider Attempt History — TODO

Status: Complete
Last updated: 2026-05-25

## M0 — History Contract

- [x] NAH-010 [owner=planner] [deps=none] [scope=docs/workstreams/addon-notification-provider-attempt-history]
  Goal: Freeze safe fields, retention size, exposure surface, and non-goals.
  Validation: `python -m json.tool docs/workstreams/addon-notification-provider-attempt-history/WORKSTREAM.json > $null`
  and `git diff --check`.
  Review: Confirm this lane does not add host persistence or provider retry.
  Evidence: `DESIGN.md`, `TODO.md`, `EVIDENCE_AND_GATES.md`.
  Result: Frozen to bounded in-memory sidecar diagnostics; persistence,
  provider retry queues, and Nako core schema changes are out of scope.
  Handoff: Continue with NAH-020.

## M1 — Bounded Recorder

- [x] NAH-020 [owner=codex] [deps=NAH-010] [scope=F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge]
  Goal: Add a bounded in-memory provider attempt recorder with redaction-safe
  entries and unit tests.
  Validation: `cargo nextest run -p nako-notification-bridge attempt_history --no-fail-fast`.
  Review: Confirm no raw request/response bodies, URLs, headers, or secrets are
  stored.
  Evidence: recorder tests.
  Result: Added bounded in-memory provider attempt recorder with redaction-safe
  entries and capacity tests.
  Handoff: Continue with NAH-030.

## M2 — Provider Send Integration

- [x] NAH-030 [owner=codex] [deps=NAH-020] [scope=F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge]
  Goal: Record provider outcomes for disabled, sent, retryable failure, and
  non-retryable failure paths.
  Validation: `cargo nextest run -p nako-notification-bridge attempt_history --no-fail-fast`
  and `cargo nextest run -p nako-notification-bridge --no-fail-fast`.
  Review: Confirm failure paths still map to host retry semantics correctly.
  Evidence: route/fixture tests.
  Result: Provider outcomes are recorded for disabled, sent, and failure paths
  without changing host retry behavior.
  Handoff: Continue with NAH-040.

## M3 — Diagnostics And Docs

- [x] NAH-040 [owner=codex] [deps=NAH-030] [scope=F:\SourceCodes\Rust\nako-official-addons\addons\notification-bridge,F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\README.md,docs/workstreams/addon-notification-provider-attempt-history]
  Goal: Expose recent attempts through redaction-safe diagnostics and update
  operator docs.
  Validation: package gate and diagnostics tests.
  Review: Confirm diagnostics output cannot leak raw event payload values or
  provider secrets.
  Evidence: diagnostics test output and docs.
  Result: Health diagnostics and operator docs now describe bounded safe
  provider attempt history.
  Handoff: Continue with NAH-050.

## M4 — Closeout

- [x] NAH-050 [owner=planner] [deps=NAH-040] [scope=docs/workstreams/addon-notification-provider-attempt-history]
  Goal: Close the lane or split persistent history into a later follow-on.
  Validation: final package gates and JSON parse.
  Review: Run review-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`.
  Result: Closed after review. Persistent history and Admin UI display are
  deferred.
  Handoff: DONE.
