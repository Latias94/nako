# Phase 4.7: Playback Session Persistence

## Goal

Persist remux and future transcode session state so playback orchestration is
observable and recoverable. Phase 4.5 and 4.6 intentionally use in-memory
session coordination; this phase moves that boundary into SQLite before HLS
and hardware acceleration add more state.

## Proposed Shape

- Add a transcode session table.
- Persist request key, source ID, session kind, output path, state,
  timestamps, and safe failure category.
- Let the remux app service create and update persisted session records.
- Reuse completed sessions from persisted state.
- Treat in-flight records from dead processes as stale through explicit
  recovery rules.
- Expose a lookup path for session status before broader playback APIs depend
  on it.

## Non-Goals

- No HLS segment persistence yet.
- No hardware acceleration queue policy yet.
- No remote source staging.
- No multi-node distributed locking.

## Validation

Expected coverage:

- remux route creates a persisted session;
- successful remux marks the session finished;
- failed remux marks the session failed with a safe error category;
- completed remux is reused after app restart;
- stale running sessions are recovered deterministically.
