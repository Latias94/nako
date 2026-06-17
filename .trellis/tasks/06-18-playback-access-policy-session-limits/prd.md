# Playback Session Admission and Limits

## Goal

Turn existing playback policy and session state into runtime admission so Nako
can cap remote playback bitrate, bound active playback sessions, and evict idle
sessions before playback work becomes unbounded.

## What We Already Know

- `PlaybackPermissionPolicy` already carries `max_streaming_bitrate` and
  `max_remote_bitrate`.
- `PlaybackSessionRecord` already stores `state`, `position_ms`,
  `duration_ms`, and `last_heartbeat_at_ms`.
- `nako-server` already owns playback session creation, heartbeat updates,
  policy checks, and runtime orchestration.
- `nako-db` already persists playback sessions in both SQLite and PostgreSQL
  adapters and has contract tests for the shared repository behavior.
- `AdminPlaybackRuntimeSettings` already exposes operator-facing playback
  runtime knobs such as concurrency and cleanup policy.
- `docs/architecture/STATE_ACCESS.md` explicitly calls out active-session
  limits and idle session termination as the next playback-access scope.
- `docs/architecture/CONTROL_PLANE.md` and ADR 0053 require this kind of work
  to stay inside the control-plane boundary, not inside ad hoc HTTP handlers.

## Assumptions

- The first slice uses a server-wide session admission ceiling, not per-user
  quotas.
- Session limit and idle timeout belong to the existing playback runtime
  settings envelope, not a new persistence table.
- The first slice can reuse the current playback session state machine and does
  not need a new public client contract.

## Requirements

- R1. Enforce `max_remote_bitrate` before creating a remote playback session
  or starting downstream playback work.
- R2. Add a server-owned active playback session limit that rejects new
  sessions once the configured ceiling is reached.
- R3. Reap idle sessions using heartbeat timestamps so stale sessions do not
  keep counting against the active limit.
- R4. Keep admission logic in the `nako-server` playback app service boundary,
  not in HTTP routes or planner helpers.
- R5. Preserve existing Direct Play, Remux, and HLS success paths when the
  configured limits are not exceeded.
- R6. Surface the new runtime knobs and current status in the admin playback
  runtime view if the implementation needs new fields.
- R7. Keep denial and cleanup outcomes redaction-safe and typed.

## Acceptance Criteria

- A remote playback request over the configured bitrate cap is denied before
  playback session creation.
- A request that would exceed the active session ceiling is denied without
  creating new playback session, transcode session, or artifact records.
- Idle sessions are terminalized or otherwise removed from active admission
  after the configured timeout and no longer count toward the limit.
- Startup or admission-time cleanup is idempotent.
- Focused `nako-server` playback tests and `nako-db` contract tests pass.
- `cargo fmt --all -- --check` and `git diff --check` pass for touched files.

## Definition of Done

- Tests added or updated for bitrate denial, session ceiling rejection, and
  idle-session eviction.
- Any new runtime settings or diagnostics are reflected in the public/admin
  contract layer.
- No unrelated dirty files are reverted or staged.
- Trellis task context validates cleanly.

## Technical Approach

- Keep bitrate limits as playback policy and implement session count / idle
  timeout as runtime admission.
- Resolve effective policy and active-session state in
  `nako-server::app::playback`, then deny before any expensive or stateful
  playback work starts.
- Use existing playback session heartbeat timestamps and state transitions to
  identify stale sessions.
- Add a bounded cleanup path inside the control-plane boundary. Prefer startup
  sweep plus admission-time reaping; only add a supervised periodic loop if the
  existing runtime already has the right home for it.
- Reuse existing SQLite/PostgreSQL playback repository functions where
  possible. Add a small repository helper only if the admission query needs a
  cheaper active-session count or a narrower active-session filter.
- Keep `nako-playback` pure. Do not move runtime cleanup or repository access
  into the planner crate.
- Avoid schema migration unless the implementation proves the existing runtime
  settings document cannot carry the new knobs safely.

## Decision (ADR-lite)

**Context**: Nako already has playback policy rows, persisted session state, and
control-plane runtime settings. The missing behavior is runtime admission and
cleanup, not another planner mode.

**Decision**: Treat bitrate caps as policy and active-session limits plus idle
timeout as server-owned runtime admission. Surface the new knobs through the
existing playback runtime settings/diagnostics path.

**Consequences**: Operators get a bounded, auditable control point for playback
cost. The implementation stays inside the control-plane boundary and avoids
turning the planner into a runtime janitor.

## Out of Scope

- Per-user quota modeling for active sessions.
- Distributed or cross-node session coordination.
- New playback planner modes.
- Public client API redesign.
- Remote worker or queue migration.
- Casting, SyncPlay, or renderer protocol changes.

## Research References

- [`research/playback-session-admission-gap.md`](research/playback-session-admission-gap.md)
  - local seam summary and implementation assumptions.

## Technical Notes

- Relevant docs:
  - `docs/architecture/STATE_ACCESS.md`
  - `docs/architecture/PLAYBACK.md`
  - `docs/architecture/CONTROL_PLANE.md`
  - `docs/adr/0039-playback-policy-and-renderer-target-boundary.md`
  - `docs/adr/0053-application-control-plane-boundary.md`
  - `docs/workstreams/playback-policy-and-renderer-targets/DESIGN.md`
- Relevant code areas:
  - `crates/nako-core/src/playback_policy.rs`
  - `crates/nako-core/src/session.rs`
  - `crates/nako-server/src/config.rs`
  - `crates/nako-server/src/app/playback/mod.rs`
  - `crates/nako-server/src/app/playback/support.rs`
  - `crates/nako-api/src/admin/playback.rs`
  - `crates/nako-db/src/sqlite/playback.rs`
  - `crates/nako-db/src/postgres/playback_runtime.rs`
  - `crates/nako-server/src/app/tests/playback.rs`
  - `crates/nako-db/src/contract_tests.rs`
- This slice should keep the existing `playback` terminology from
  `CONTEXT.md`.

## Implementation Units

- U1. Surface the runtime admission knobs and diagnostics.
  - Goal: add the active-session ceiling and idle timeout to the existing
    playback runtime settings path.
  - Test focus: default values, validation, and round-trip reporting.
- U2. Enforce bitrate caps and session admission in the playback app service.
  - Goal: deny remote playback above the configured bitrate cap, reject new
    sessions when the ceiling is reached, and reap idle sessions before
    admission.
  - Test focus: denied remote start, ceiling rejection, stale-session cleanup,
    and unchanged happy paths.
- U3. Tighten repository and app coverage.
  - Goal: prove SQLite/PostgreSQL playback repositories and server playback
    tests agree on the new admission behavior.
  - Test focus: heartbeat freshness, stale-session terminalization, and no
    orphaned session records on denial.
