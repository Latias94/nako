# HLS Runtime Lifecycle Boundary Closeout

Status: Closed
Closed: 2026-05-31
Task: HRLB-040

## Closeout Claim

The HLS runtime lifecycle boundary is frozen, covered by focused
behavior-preserving tests, and closed with explicit follow-on splits. The final
HLS gate was initially blocked by progressive readiness test instability; that
scope was split to HPRTS, fixed, verified, and then HRLB closeout was retried
successfully.

## Delivered

- HLS lifecycle invariant table and cleanup ownership map.
- Behavior-preserving HLS lifecycle coverage for timeout cleanup, stale startup
  recovery, and remote staged-input release.
- Follow-on splits for HLS test stability, artifact I/O pressure, resource
  admission queueing, remote workers, LL-HLS/CMAF, and player UX.
- HPRTS follow-on completion and final HRLB closeout verification.

## Validation

```text
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json
python -m json.tool docs/workstreams/hls-progressive-readiness-test-stability/WORKSTREAM.json
git diff --check
```

Result: passed on 2026-05-31. The full HLS gate ran 71 tests, all passed, with
26 slow tests.

## Follow-Ons

- `proposed:hls-artifact-io-pressure-enforcement`
- `proposed:playback-admission-queueing-and-waitlist`
- `proposed:remote-transcode-worker-runtime`
- `proposed:ll-hls-cmaf-runtime`
- `proposed:player-hls-session-controls-and-recovery`
- optional future HLS suite scheduling or nextest grouping work if Windows
  process-backed tests continue to grow slower
