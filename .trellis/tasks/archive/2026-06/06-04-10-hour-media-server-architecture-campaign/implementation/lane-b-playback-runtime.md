# Lane B: Playback Transcode Runtime Session Module

Date: 2026-06-04

## Scope

Behavior-preserving extraction of shared playback/transcode session lifecycle
helpers in `crates/nako-server/src/app/playback/`.

## Changed Files

- `crates/nako-server/src/app/playback/runtime_session.rs`
- `crates/nako-server/src/app/playback/mod.rs`
- `crates/nako-server/src/app/playback/hls_flow.rs`
- `crates/nako-server/src/app/playback/remux_flow.rs`
- `crates/nako-server/src/app/tests/playback.rs`

## What Changed

- Added a server-owned runtime session helper for:
  - active/latest transcode session lookup by `(source_id, kind, request_key)`
  - finished-session reuse when the expected output exists
  - playback-session to transcode-session linkage
  - typed HLS/Remux bound-session validation
  - redaction-safe helper errors for missing/cancelled/failed sessions
- Kept HLS and Remux mode-specific orchestration in their flow modules.
- Kept HTTP handlers, FFmpeg argv planning, and pure playback decisions out of
  the helper.
- Added a small regression assertion that the remux playback session returned
  by the app layer carries the linked transcode session id.

## Validation

- `cargo fmt --all` - passed
- `cargo check -p nako-server --tests` - passed
- `cargo check -p nako-playback -p nako-transcode -p nako-server --tests` - passed
- `cargo nextest run -p nako-server hls --no-fail-fast` - passed
- `cargo nextest run -p nako-server remux --no-fail-fast` - passed

## Notes

- Scope-out dirty files from other lanes were present in the worktree and left
  untouched.
- No public DTO, schema, or planner contract changed in this lane.
