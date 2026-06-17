# Playback Session Strict Reservation

## Goal

Make Nako's server-wide active playback session ceiling a strict runtime
admission boundary for self-hosted deployments, not a best-effort
count-then-create check.

## What We Already Know

- The previous task shipped runtime admission for remote playback permission,
  `max_remote_bitrate`, server-wide active session limits, and idle session
  reaping.
- The remaining risk is concurrency: admission currently counts active
  sessions before creating the playback session, so simultaneous starts can
  exceed the limit.
- Remux, HLS, and renderer flows need admission before expensive downstream
  work starts, but some of them create the linked playback session only after
  transcode/session setup.
- `PlaybackSessionRepository` is shared by SQLite and PostgreSQL adapters and
  already has contract tests.
- `.trellis/spec/nako-server/backend/quality-guidelines.md` now records the
  playback session admission contract and explicitly calls out strict
  reservation as a follow-on.

## Requirements

- R1. Add a durable reservation or atomic admission path so active playback
  session ceiling cannot be exceeded by concurrent session starts on one
  database.
- R2. Keep admission inside the playback app-service and repository boundary,
  not inside HTTP routes or planner crates.
- R3. Preserve early denial before expensive Remux/HLS/renderer work starts.
- R4. Reap idle active sessions before evaluating capacity, with SQLite and
  PostgreSQL parity.
- R5. Ensure failed downstream startup does not leave a permanent active slot
  that blocks future playback.
- R6. Keep denial errors typed and redaction-safe: session ceiling remains
  `Conflict`, policy denial remains `Forbidden`.
- R7. Preserve existing Direct Play, Remux, HLS, renderer, and browser-ticket
  success paths when capacity is available.

## Acceptance Criteria

- A concurrent session-start test with a configured limit of `1` creates at
  most one active playback session.
- A rejected concurrent request creates no transcode session or artifact rows.
- Idle stale sessions are ended before reservation and do not consume capacity.
- A reservation/session creation failure releases or terminalizes its slot.
- SQLite contract coverage proves the atomic behavior; PostgreSQL adapter code
  has matching SQL and ignored contract coverage.
- Focused `nako-server` playback tests pass.
- `cargo fmt --all -- --check`, relevant `cargo nextest` gates, and
  `git diff --check` pass.

## Definition of Done

- Repository trait shape and adapter SQL make the atomic boundary explicit.
- App-service entry points use the strict boundary consistently.
- Trellis spec/architecture docs are updated if the implementation creates a
  reusable pattern beyond the existing admission contract.
- No unrelated dirty files are reverted or staged.

## Technical Approach

Prefer a repository-owned atomic admission helper that performs:

1. idle session reaping;
2. active count inside the same transaction or write lock;
3. creation of a reserved/active playback session or a typed capacity denial.

Direct playback can create the session immediately inside the atomic helper.
Remux/HLS/renderer flows may need either:

- a pre-created session that is linked to downstream work after startup; or
- a durable reservation record represented by a playback session state that is
  terminalized if downstream work fails.

The implementation should choose the smallest shape that preserves the existing
session state model and avoids a new table unless the current model cannot
represent reserved capacity safely.

## Decision (ADR-lite)

**Context**: Runtime admission shipped as count-then-create, which is correct
for serial startup but not strict under concurrent starts.

**Decision**: Move capacity reservation into repository-backed atomic behavior
instead of adding process-local mutexes. Process-local locks would not protect
multi-process or PostgreSQL deployments and would still be awkward around
Remux/HLS pre-admission.

**Consequences**: The database becomes the source of truth for session capacity.
The first strict slice may keep server-wide limits only; per-user quotas remain
future scope.

## Out of Scope

- Per-user or per-household session quotas.
- Distributed multi-node control-plane coordination beyond one shared database.
- Queueing or waitlist behavior for playback starts.
- New public client API shape.
- Playback planner changes.

## Technical Notes

- Relevant docs:
  - `docs/architecture/STATE_ACCESS.md`
  - `docs/architecture/PLAYBACK.md`
  - `docs/architecture/CONTROL_PLANE.md`
  - `docs/adr/0039-playback-policy-and-renderer-target-boundary.md`
  - `docs/adr/0053-application-control-plane-boundary.md`
  - `.trellis/spec/nako-server/backend/quality-guidelines.md`
- Relevant code:
  - `crates/nako-core/src/repository/playback_session.rs`
  - `crates/nako-db/src/sqlite/playback.rs`
  - `crates/nako-db/src/postgres/playback_runtime.rs`
  - `crates/nako-db/src/contract_tests.rs`
  - `crates/nako-server/src/app/playback/mod.rs`
  - `crates/nako-server/src/app/playback/remux_flow.rs`
  - `crates/nako-server/src/app/playback/hls_flow.rs`
  - `crates/nako-server/src/app/playback/renderer_flow.rs`
  - `crates/nako-server/src/app/tests/playback.rs`
