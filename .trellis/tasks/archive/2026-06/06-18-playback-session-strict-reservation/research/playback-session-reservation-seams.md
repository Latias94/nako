# Playback Session Reservation Seams

## Local Findings

- `PlaybackSessionRepository` owns persisted session creation and state updates
  for SQLite/PostgreSQL.
- Runtime admission currently calls idle reap and active count before session
  creation in the playback app service.
- Direct playback can create a session immediately; Remux/HLS/renderer flows
  need a capacity claim before downstream startup and then link the session to
  transcode/runtime state.

## Design Constraints

- Do not add raw SQL to `nako-server`; strict capacity belongs in `nako-db`.
- Do not use a process-local mutex as the durable solution. It would not cover
  multi-process or PostgreSQL deployments.
- Avoid a new table unless the existing session state model cannot represent
  pre-start capacity safely.
- Reaping and capacity check must happen in one database-owned critical section
  with session creation/reservation.

## Feasible Implementation Options

### Option A: Atomic Create With Existing Session State

Repository method accepts a `NewPlaybackSessionRecord`, performs idle reap,
counts active sessions, and inserts the session in one transaction/write lock.

Pros:
- Smallest API surface.
- Directly proves active rows never exceed limit.
- Keeps all capacity evidence in existing playback session table.

Cons:
- Remux/HLS may create a session before downstream startup and must terminalize
  it safely if startup fails.

### Option B: Dedicated Reserved Session State

Add a `reserved` state and count it as active capacity until it becomes active
or failed.

Pros:
- Models pre-start capacity explicitly.
- Cleaner for long Remux/HLS startup windows.

Cons:
- Requires schema/domain/state changes and more API compatibility review.

### Option C: Separate Reservation Table

Create a playback session reservation table with expiry and release semantics.

Pros:
- Strong modeling for future queueing/waitlists.

Cons:
- Larger migration and cleanup surface; likely overbuilt for first strict
  server-wide limit slice.

## Recommendation

Start with Option A. It enforces strict capacity through the existing persisted
session table and is easiest to test across SQLite/PostgreSQL. If Remux/HLS
failure semantics become too contorted, split Option B into a follow-on with an
explicit state-machine migration.
