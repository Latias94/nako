# Playback Session Admission Gap

## Summary

The current codebase already has most of the structural pieces needed for
runtime playback admission, but not the final enforcement loop.

## Findings

- `PlaybackPermissionPolicy` already models bitrate caps:
  `max_streaming_bitrate` and `max_remote_bitrate` exist in `nako-core`.
- `PlaybackSessionRecord` already stores the data needed for idle-session
  eviction: `state`, `position_ms`, `duration_ms`, and
  `last_heartbeat_at_ms`.
- `nako-server::app::playback` already owns session creation and heartbeat
  writes, so the admission boundary belongs there.
- `nako-db` already exposes shared SQLite/PostgreSQL playback session
  repository behavior and contract tests.
- `nako-server::config::PlaybackConfig` already has operator-facing playback
  runtime knobs, so the first slice can likely extend the existing runtime
  settings envelope instead of adding a new table.
- I did not find an existing explicit active-session limit or idle timeout
  setting in the current repository search.

## Implementation Implication

The simplest reliable first slice is:

1. keep bitrate caps as playback policy;
2. add a runtime admission ceiling and idle timeout to playback runtime
   settings;
3. use playback session heartbeats and state transitions to reap stale
   sessions before admission;
4. keep all cleanup inside the server control-plane boundary.

## Reference Paths

- `crates/nako-core/src/playback_policy.rs`
- `crates/nako-core/src/session.rs`
- `crates/nako-server/src/app/playback/mod.rs`
- `crates/nako-server/src/app/playback/support.rs`
- `crates/nako-server/src/config.rs`
- `crates/nako-api/src/admin/playback.rs`
- `crates/nako-db/src/sqlite/playback.rs`
- `crates/nako-db/src/postgres/playback_runtime.rs`
