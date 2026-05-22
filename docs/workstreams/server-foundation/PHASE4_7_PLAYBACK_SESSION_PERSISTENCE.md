# Phase 4.7: Playback Session Persistence

Status: completed.

## Goal

Persist remux and future transcode session state so playback orchestration is
observable and recoverable. Phase 4.5 and 4.6 intentionally use in-memory
session coordination; this phase moves that boundary into SQLite before HLS
and hardware acceleration add more state.

## Proposed Shape

- Added a `transcode_sessions` SQLite table.
- Persisted request key, source ID, session kind, output path, state,
  timestamps, and safe failure category/message.
- Added `TranscodeSessionRepository` in `nako-core` with `nako-db`
  implementation.
- Moved shared transcode session ID/kind/state domain types into `nako-core`
  while keeping FFmpeg execution details in `nako-transcode`.
- Let the remux app service create and update persisted session records.
- Reuse completed sessions from persisted state when the staged output still
  exists.
- Treat active records from dead processes as stale during server startup.
- Expose `GET /playback/sessions/{session_id}` for session status lookup.

## Non-Goals

- No HLS segment persistence yet.
- No hardware acceleration queue policy yet.
- No remote source staging.
- No multi-node distributed locking.

## Validation

Coverage:

- remux route creates a persisted session;
- successful remux marks the session finished;
- failed remux marks the session failed with a safe error category;
- completed remux is reused after app restart;
- stale running sessions are recovered deterministically;
- playback session lookup returns the persisted state.
