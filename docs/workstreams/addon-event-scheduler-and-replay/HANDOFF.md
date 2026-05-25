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

- Task ID: AESR-030
- Owner: codex
- Files:
  - `crates/nako-core`
  - `crates/nako-db`
  - `crates/nako-server`
- Validation:
  - `cargo nextest run -p nako-server addon_event_scheduler --no-fail-fast`
- Status: READY
- Review: AESR-020 due work selection passed focused gates; AESR-030 still
  needs durable in-flight guards and automatic retry.
- Evidence: AESR-020 evidence is recorded in `EVIDENCE_AND_GATES.md`.

## Decisions Since Last Update

- Normal scheduler delivery must skip already succeeded addon/event/subscription
  tuples.
- Forced replay must be explicit and separate from normal delivery.
- Scheduler diagnostics must not expose outbox payload values.
- AESR-020 stores scheduler work as redaction-safe routing/attempt facts in the
  repository and computes due/deferred/retry state in the server layer, where
  manifest max attempts and current grants are available.
- Event subscription filters should execute before sidecar calls, unless filter
  language complexity forces an ADR split.

## Blockers

- `cargo fmt --all -- --check` is currently blocked by unrelated parallel
  scan-addon workstream edits in server files. Do not format or revert those
  files from this lane unless that workstream owner agrees.

## Next Recommended Action

Implement AESR-030 by adding a durable in-flight guard/lease for the
addon/event/subscription tuple and consuming `next_retry_at` for automatic
retry selection. Do not start the background scheduler loop until duplicate
delivery behavior is proven.
