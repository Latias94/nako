# Phase 4.8: HLS Transcode Foundation

Status: completed.

## Goal

Add the first HLS transcode foundation on top of persisted playback sessions.
This phase should prove playlist and segment lifecycle management before
hardware acceleration policy is introduced.

## Proposed Shape

- Added HLS output layout planning under the remux/transcode staging root.
- Modeled playlist and segment paths as session-owned artifacts.
- Added FFmpeg HLS command planning without enabling hardware acceleration.
- Added an app-service boundary for starting or reusing an HLS session.
- Added HTTP routes for media playlists and segment bytes.
- Reused the existing `transcode_sessions` table for lifecycle state.
- Kept duplicate request behavior based on persisted active sessions.

## Non-Goals

- No VAAPI, NVENC, or QuickSync policy yet.
- No adaptive bitrate ladder beyond a minimal single-variant foundation.
- No remote source staging/cache behavior.
- No multi-node distributed locking.

## Validation

Coverage:

- HLS session creation persists a playback session;
- playlist and segment paths cannot escape the staging root;
- completed or active sessions are reused deterministically;
- stale sessions are recovered through the existing startup rule;
- failed FFmpeg HLS runs retain a safe failure category/message;
- HTTP playlist and segment routes map missing or unsatisfiable artifacts to
  stable API errors.
