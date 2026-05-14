# 0014: Use a Durable Event Outbox for Webhooks and Automation

## Status

Proposed

## Context

Taru now has a local media-library MVP with scans, metadata refreshes, NFO
jobs, catalog graph updates, playback routes, and persisted playback sessions.
The next extension surface needs to react to those changes without coupling
domain mutations directly to webhook HTTP calls or automation provider calls.

Inline delivery would make user-facing operations slower and less reliable.
In-memory event queues would lose work across restarts. A durable event boundary
is needed before webhook delivery, addon notifications, or automation jobs can
be trusted.

## Decision

Persist domain events into an event outbox table before implementing delivery.

The outbox record should contain:

- stable event ID;
- event type, for example `library.scanned`, `item.metadata_refreshed`,
  `nfo.imported`, `playback.session_finished`, or `user.tagged_item`;
- subject type and subject ID;
- optional library ID and source ID;
- idempotency key;
- occurred timestamp;
- safe `payload_json`;
- dispatch status and attempt metadata once delivery is implemented.

Event writes should happen in the same transaction as the domain mutation when
the repository boundary supports it. When a single transaction is not practical,
the producing service must make event creation idempotent by event type,
subject, and idempotency key.

Outbox payloads must not contain plaintext secrets, local filesystem paths, or
large binary data. They should prefer stable Taru IDs and small public snapshots
that downstream consumers can use to fetch current state.

Consumers of the outbox are separate pipeline owners:

- webhook delivery worker;
- automation job scheduler;
- future addon notification bridge;
- future audit/export tools.

M5.1 should implement event persistence and event write points only. M5.2 and
later phases may add delivery and automation consumers.

## Consequences

- Domain writes stay fast and do not depend on external HTTP availability.
- Webhook and automation delivery can be retried after restart.
- Event payload shape becomes a public-ish contract and needs versioning care.
- Repository transaction boundaries become more important.
- Tests must verify idempotent event creation and no plaintext secrets in event
  payloads.

## Alternatives Considered

- Call webhooks inline from domain services: rejected because external latency
  and failures would leak into core workflows.
- Use an in-memory queue first: rejected because webhook and automation work
  must survive restarts.
- Adopt a message broker immediately: deferred because Taru is still a modular
  monolith and SQLite-backed outbox is enough for the self-hosted MVP.

## Related Workstreams

- `docs/workstreams/addons-automation/`
- `docs/workstreams/server-foundation/`
