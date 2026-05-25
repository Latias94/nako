# Addon Event Scheduler And Replay — Handoff

Status: Complete
Last updated: 2026-05-25

## Current State

This workstream is closed. Nako now has operational Addon Event scheduling:
due work discovery, automatic retry through durable `next_retry_at`, in-flight
deduplication through delivery claims, disabled-by-default supervised scheduler
lifecycle integration, explicit forced replay with operator intent, and
redaction-safe scheduler diagnostics with simple host-side event fact filters.

## Active Task

- Task ID: none
- Owner: planner
- Files:
  - `docs/workstreams/addon-event-scheduler-and-replay`
- Validation:
  - see `EVIDENCE_AND_GATES.md`
- Status: DONE
- Review: AESR-060 closeout found no blocking workstream-compliance or
  code-quality findings.
- Evidence: AESR-020 through AESR-060 evidence is recorded in
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
- AESR-050 adds `POST /admin/v1/events/{event_id}/addon-events/replay` with a
  required operator `reason_code`. Replays write a new delivery attempt with
  `forced_replay=true` and `replay_reason_code`, while normal delivery still
  skips already succeeded addon/event/subscription tuples.
- Host-side subscription filters now evaluate simple JSON event facts before
  durable claims and sidecar calls. Supported facts are event kind, subject
  kind/id, `library_id`, and `source_id`. Non-matches are reported with the
  redaction-safe `filter_not_matched` reason.

## Blockers

- None currently known.

## Follow-Ons

- Open notification bridge as a separate workstream before implementing
  provider fan-out.
- Keep watch-state sync, MCP media steward, Arr-stack integration,
  DLNA/UPnP/WebDAV compatibility, and Network Tunnel Provider behavior as
  separate lanes with their own gates and ADRs when they become active.
- Do not add hidden scheduler/replay scope back into this closed lane.
