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

- Task ID: AESR-040
- Owner: codex
- Files:
  - `crates/nako-server`
- Validation:
  - `cargo nextest run -p nako-server addon_event_scheduler --no-fail-fast`
- Status: READY
- Review: AESR-030 durable claim and automatic retry passed focused gates; the
  next slice should add the background scheduler lifecycle without changing
  replay semantics.
- Evidence: AESR-020 and AESR-030 evidence is recorded in
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

## Blockers

- `cargo fmt --all -- --check` is currently blocked by unrelated parallel
  scan-addon workstream edits in server files. Do not format or revert those
  files from this lane unless that workstream owner agrees.

## Next Recommended Action

Implement AESR-040 by wiring a bounded scheduler loop into server runtime
lifecycle. Reuse the existing due-work diagnostics and durable claim API; do not
start forced replay, filters, or notification bridge work in the same slice.
