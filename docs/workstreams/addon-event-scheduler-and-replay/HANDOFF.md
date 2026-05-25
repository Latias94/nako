# Addon Event Scheduler And Replay — Handoff

Status: Active
Last updated: 2026-05-25

## Current State

This workstream is opened as the immediate follow-on to Addon Ecosystem
Foundation. Manual Addon Event Delivery exists and the official metadata
scraper has a minimal `library.scanned` event proof. What remains is
operational scheduling: due work discovery, automatic retry, in-flight
deduplication, explicit forced replay, and redaction-safe diagnostics.

## Active Task

- Task ID: AESR-050
- Owner: codex
- Files:
  - `crates/nako-api`
  - `crates/nako-server`
  - `crates/nako-db`
- Validation:
  - `cargo nextest run -p nako-server addon_event_replay addon_event_filter --no-fail-fast`
- Status: READY
- Review: AESR-040 scheduler lifecycle passed focused gates; the next slice
  should keep forced replay explicit and evaluate subscription filters before
  sidecar calls.
- Evidence: AESR-020 through AESR-040 evidence is recorded in
  `EVIDENCE_AND_GATES.md`.

## Decisions Since Last Update

- Normal scheduler delivery must skip already succeeded addon/event/subscription
  tuples.
- Forced replay must be explicit and separate from normal delivery.
- Scheduler diagnostics must not expose outbox payload values.
- AESR-020 stores scheduler work as redaction-safe routing/attempt facts in the
  repository and computes due/deferred/retry state in the server layer, where
  manifest max attempts and current grants are available.
- AESR-030 adds `claim_addon_event_delivery_attempt` as the durable execution
  boundary. The claim writes a `running` attempt with `lease_expires_at`; active
  leases suppress duplicate sidecar calls, expired leases allow the next attempt,
  succeeded attempts suppress normal delivery, and failed attempts only retry
  once `next_retry_at` is due.
- Addon Event delivery now claims before taking the sidecar execution semaphore,
  so a queued worker still leaves a durable in-flight fact for concurrent
  schedulers to observe.
- Event subscription filters should execute before sidecar calls, unless filter
  language complexity forces an ADR split.
- AESR-040 adds a disabled-by-default `addon_event_scheduler` config block and
  starts a supervised `addon_event_scheduler` runtime task only when enabled.
  The loop scans pending outbox events in bounded batches, checks due/retry-due
  scheduler work, and dispatches through the existing durable delivery path with
  configured event concurrency.
- Startup diagnostics now expose whether the scheduler runtime task was started.

## Blockers

- None currently known for AESR-050.

## Next Recommended Action

Implement AESR-050 by adding explicit forced replay and event subscription
filter evaluation. Keep replay separate from normal scheduled delivery, keep
payload values out of diagnostics, and do not start notification bridge work in
this lane.
