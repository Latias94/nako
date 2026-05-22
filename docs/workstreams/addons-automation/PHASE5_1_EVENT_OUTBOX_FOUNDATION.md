# Phase 5.1: Event Outbox Foundation

Status: completed.

## Goal

Persist domain events and outbox state so webhook, automation, addon
notification, and audit consumers can be added without inline external HTTP
calls.

## Completed Shape

- Added domain event kinds, subjects, outbox status, new-event input records,
  persisted event records, and `EventOutboxRepository` in `nako-core`.
- Added SQLite migration `0009_event_outbox.sql`.
- Implemented idempotent `SqliteStore` outbox enqueue, lookup, idempotency-key
  lookup, and paginated listing.
- Re-exported the shared core event types from `nako-events` so future event
  pipeline code uses the same domain contract.
- Wrote outbox events after successful library scan, metadata refresh, NFO
  import, NFO export, and remux/HLS playback session completion.
- Kept webhook delivery out of this phase.

## Event Types

Initial event kinds:

- `library.scanned`
- `item.metadata_refreshed`
- `nfo.imported`
- `nfo.exported`
- `playback.session_finished`

Payloads are small JSON snapshots containing stable Nako IDs, status, and
counts. They intentionally avoid plaintext secrets, raw local paths, and binary
data.

## Non-Goals

- No webhook endpoint configuration.
- No delivery attempts or retry worker.
- No webhook signing.
- No automation provider job execution.
- No addon registration or resource calls.

## Validation

Coverage:

- `nako-db` tests verify durable outbox persistence, idempotent enqueue by
  event kind and idempotency key, status defaults, and payload safety checks.
- `nako-server` tests verify scan, NFO, remux, and HLS success paths create
  outbox events without raw local paths.
- metadata event helper coverage verifies provider payloads do not include
  secret references or raw local paths.
- workspace gates pass: `cargo fmt --all -- --check`, `cargo check
  --workspace`, `cargo nextest run --workspace`, and `git diff --check`.
