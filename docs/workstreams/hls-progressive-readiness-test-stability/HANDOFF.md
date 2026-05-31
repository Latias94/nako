# HLS Progressive Readiness Test Stability - Handoff

Status: Closed
Last updated: 2026-05-31

## Current State

This follow-on is closed.

`HPRTS-010` split the full HLS gate failure from `HRLB-040` after the
progressive readiness tests failed twice under default nextest concurrency but
passed individually.

`HPRTS-020` classified the failure as Windows full-suite process-backed test
timing. The two target tests returned after the original fixed 60s guard in the
default suite. The fix is test-only and behavior-preserving: both tests now use
a named process-backed playlist readiness timeout that stays 60s off Windows
and is 180s on Windows.

`HPRTS-030` reran final gates. The default full HLS gate passed, so this
follow-on unblocked `HRLB-040` closeout.

## Completed

- Task ID: `HPRTS-010`, `HPRTS-020`, `HPRTS-030`
- Lane: `playback-transcode`
- Final status: DONE_WITH_CONCERNS

## Validation

Fresh planner verification on 2026-05-31:

```text
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
python -m json.tool docs/workstreams/hls-progressive-readiness-test-stability/WORKSTREAM.json
python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json
git diff --check
```

Result: passed. The full HLS gate ran 71 tests, all passed, with 26 slow tests.

## Residual Risks

- The full HLS gate still has many process-backed tests above 60s on Windows.
- Broader fixture scheduling or nextest grouping should be a separate follow-on
  if the suite continues to slow down.
- PAIP artifact I/O pressure, resource admission unification, remote workers,
  LL-HLS/CMAF, player UX, DTO changes, schema changes, and VFS behavior changes
  remain out of scope.

## Next Recommended Action

Use the final HPRTS evidence to close `HRLB-040`. Do not reopen this follow-on
unless the same progressive readiness tests regress again under the default
full HLS gate.
