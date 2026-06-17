# Playback Write Hotspots Research

## What I found

- Playback heartbeat writes happen every `30_000ms` in `web/src/features/media/video-player.tsx`.
- The web player sends heartbeat updates for `active`, `paused`, `ended`, and
  `failed` states.
- `nako-server` persists playback session heartbeats through
  `PlaybackAppService::record_playback_session_heartbeat`.
- The DB heartbeat update is a single row update guarded by terminal-state
  filtering, then a follow-up read.
- HLS transcode runtime metrics are written once after runner startup when the
  outcome reports metrics; that path already skips empty metrics and logs
  failures without failing playback.
- Renderer session heartbeats are also single-row updates, but they are not the
  playback heartbeat hot path.
- Durable job leases use a 10s heartbeat interval in `app/job_runtime.rs`, but
  that is generic control-plane infrastructure, not a playback-specific loop.

## What this means

The actual hot write cadence is modest. Playback does not currently look like a
candidate for aggressive write coalescing. The better first slice is to prove
that the existing cadence stays healthy under concurrent playback and scan/job
writes.

## Recommendation

Prioritize pressure tests and operational expectations. If a test exposes real
contention, then consider a later heartbeat coalescing slice for playback state
only.
