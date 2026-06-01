# PTJCH-220 - Playback Runtime

Status: Done
Date: 2026-06-01

## Summary

- Moved HLS supersede candidate discovery and cancellation request behavior
  into playback runtime control helpers.
- Kept HLS runtime admission in `nako-server` while leaving FFmpeg command
  planning and HLS artifact allow-list ownership in `nako-transcode`.
- Added bounded supersede admission retry so a seek/restart can cancel the
  replaced local HLS runner and wait briefly for its CPU/GPU permit to release.
- Synchronized playback-session state after HLS supersede: active HLS playback
  sessions linked to superseded transcode sessions are marked cancelled.
- Added a regression test for a running HLS playlist session occupying the only
  CPU transcode slot, followed by a seeked HLS request that must supersede it
  without failing on `cpu_transcode` admission.
- Kept `cancel_requested` HLS sessions in the supersede candidate set because
  they are still active and may still hold the local runner permit until the
  cancellation registry is signalled.
- Aligned the system playback runtime active-pressure readiness wait with the
  existing process-backed HLS timeout helper pattern so the full Windows gate
  does not fail before fake FFmpeg has produced the first playlist.

## Validation

```text
cargo nextest run -p nako-server hls_playlist_playback_seek_supersedes_running_session_without_admission_dead_end --no-fail-fast
cargo nextest run -p nako-server hls_playlist_playback_seek_waits_for_cancel_requested_runner_permit --no-fail-fast
cargo nextest run -p nako-server hls_playlist_playback hls_source_seek_generation_supersedes_active_prior_generation --no-fail-fast
cargo nextest run -p nako-server admin_v1_playback_runtime_reports_active_resource_pressure --no-fail-fast
cargo nextest run -p nako-server hls playback --no-fail-fast
cargo fmt --all -- --check
python -m json.tool docs/workstreams/playback-transcode-jellyfin-class-hardening/WORKSTREAM.json
git diff --check
```

Result: passed on 2026-06-01. The tracer test failed red first with
`playback resource cpu_transcode is busy`, then passed after the runtime fix.
The focused HLS playlist/seek gate ran 4 tests and passed. The focused active
resource-pressure diagnostics test passed. Integration verification added and
passed the `cancel_requested` runner permit regression. The required
`nako-server` `hls playback` gate ran 153 tests and passed on the final rerun.
`cargo fmt --all -- --check`, `python -m json.tool`, and `git diff --check`
passed; `git diff --check` emitted LF/CRLF working-copy warnings only.

## Remaining Risk

- Supersede admission is still a bounded retry, not durable queueing or a full
  playback waitlist. Queueing remains a separate follow-on.
- HLS artifact I/O pressure is intentionally unchanged and remains for
  `PTJCH-310` or a PAIP follow-on.
