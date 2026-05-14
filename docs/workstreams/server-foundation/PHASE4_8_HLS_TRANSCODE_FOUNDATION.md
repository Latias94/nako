# Phase 4.8: HLS Transcode Foundation

## Goal

Add the first HLS transcode foundation on top of persisted playback sessions.
This phase should prove playlist and segment lifecycle management before
hardware acceleration policy is introduced.

## Proposed Shape

- Add HLS output layout planning under the remux/transcode staging root.
- Model playlist and segment paths as session-owned artifacts.
- Add FFmpeg HLS command planning without enabling hardware acceleration yet.
- Add an app-service boundary for starting or reusing an HLS session.
- Add HTTP routes for HLS master/media playlists and segment bytes.
- Use the existing `transcode_sessions` table for session lifecycle state.
- Keep duplicate request behavior based on persisted active sessions.

## Non-Goals

- No VAAPI, NVENC, or QuickSync policy yet.
- No adaptive bitrate ladder beyond a minimal single-variant foundation.
- No remote source staging/cache behavior.
- No multi-node distributed locking.

## Validation

Expected coverage:

- HLS session creation persists a playback session;
- playlist and segment paths cannot escape the staging root;
- completed or active sessions are reused deterministically;
- stale sessions are recovered through the existing startup rule;
- failed FFmpeg HLS runs retain a safe failure category/message;
- HTTP playlist and segment routes map missing or unsatisfiable artifacts to
  stable API errors.
