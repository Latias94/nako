# HPRTS-020 - Stabilize Progressive Readiness

Date: 2026-05-31
Status: Done with concerns

## Summary

The default full HLS gate failure was classified as Windows process-backed test
timing under nextest concurrency. The two progressive readiness tests passed
individually and with reduced `hls` gate concurrency, but in the default gate
they returned after the original 60s test-local guard.

The fix is test-only. Both target tests now use a named process-backed playlist
readiness timeout helper. The helper keeps the guard at 60s off Windows and
uses 180s on Windows to cover the observed full-suite process startup tail.

No production HLS runtime behavior, DTO, schema, VFS, PAIP, LL-HLS, remote
worker, or player UX behavior changed.

## Evidence

```text
cargo nextest run -p nako-server app::tests::playback::hls_playlist_playback_returns_when_playlist_is_ready_before_runner_finishes --no-fail-fast
```

Result: passed after the change, 1/1 in 10.748s.

```text
cargo nextest run -p nako-server http::tests::playback::hls_playlist_route_returns_while_transcode_session_is_running --no-fail-fast
```

Result: passed after the change, 1/1 in 6.702s.

```text
cargo nextest run -p nako-server hls --no-fail-fast
```

Result: passed after the change, 71/71 in 116.102s. The two target tests
passed inside the default gate in 69.218s and 71.187s.

## Residual Concern

The default full HLS gate still reports many process-backed tests above 60s on
Windows. This task unblocks HRLB closeout by making the two progressive
readiness assertions align with the observed suite timing, but broader fixture
scheduling or nextest grouping should remain a separate planner decision if the
suite continues to grow slower.
