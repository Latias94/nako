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

- Task ID: AESR-020
- Owner: unassigned
- Files:
  - `crates/nako-core`
  - `crates/nako-db`
  - `crates/nako-server`
- Validation:
  - `cargo nextest run -p nako-db addon_event_scheduler --no-fail-fast`
  - focused server scheduler diagnostics tests
- Status: READY
- Review: pending
- Evidence: AESR-010 scope is recorded in this workstream.

## Decisions Since Last Update

- Normal scheduler delivery must skip already succeeded addon/event/subscription
  tuples.
- Forced replay must be explicit and separate from normal delivery.
- Scheduler diagnostics must not expose outbox payload values.
- Event subscription filters should execute before sidecar calls, unless filter
  language complexity forces an ADR split.

## Blockers

- None currently.

## Next Recommended Action

Implement AESR-020 due work selection before adding a background scheduler loop.
