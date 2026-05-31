# HLS Progressive Readiness Test Stability

Status: Closed
Last updated: 2026-05-31

## Why This Lane Exists

`HRLB-040` attempted to close the HLS runtime lifecycle boundary, but the
required full HLS gate failed twice:

```text
cargo nextest run -p nako-server hls --no-fail-fast
```

Both failures were progressive readiness tests that passed when run
individually. The full-suite failure blocks HRLB closeout because agents cannot
trust the HLS gate while it is load-sensitive.

## Relevant Authority

- `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`
- `docs/adr/0005-bounded-async-pipelines-and-resource-budgets.md`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/LANES.md`
- `docs/workstreams/hls-runtime-lifecycle-boundary/`
- `docs/workstreams/hls-progressive-runtime-boundary/`

## Problem

Progressive playlist readiness is expected to return a playlist before the
FFmpeg runner exits once the playlist contains a media or variant URI line. In
the default full HLS nextest run, two readiness tests time out around 60
seconds even though the same tests pass individually:

- `app::tests::playback::hls_playlist_playback_returns_when_playlist_is_ready_before_runner_finishes`
- `http::tests::playback::hls_playlist_route_returns_while_transcode_session_is_running`

This creates false negative gates and masks real HLS lifecycle regressions.

## Target State

When this workstream closes:

- the default full HLS gate passes without relying on individual reruns;
- the root cause is classified as test harness timing, fixture contention, or a
  real runtime bug;
- any fix is behavior-preserving unless the planner approves a runtime change;
- HRLB-040 can be rerun with trustworthy final gate evidence.

## In Scope

- Progressive readiness test instrumentation and fixture review.
- HLS app and HTTP test harness timing, polling, and fake runner behavior.
- Behavior-preserving changes to tests or local test helpers.
- Focused diagnostics that explain why suite concurrency changes timing.

## Out Of Scope

- PAIP artifact I/O pressure enforcement.
- Resource admission queueing or waitlists.
- Remote transcode workers.
- LL-HLS/CMAF, DASH/CMAF, DRM, or key delivery.
- Player UX or client controls.
- Public/Admin DTO changes.
- Storage schema changes.
- VFS behavior changes.
- New HLS runtime behavior without planner approval.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The failure is load-sensitive rather than a deterministic functional failure. | High | Both failed tests passed individually after two full-suite failures. | If false, stop and report the runtime bug boundary to planner before changing behavior. |
| The first slice should stabilize tests before PAIP or LL-HLS work starts. | High | HRLB-030 and HRLB-040 evidence. | Larger runtime work will keep producing ambiguous gate failures. |
| DTO, schema, and VFS behavior are not needed to stabilize this gate. | Medium | Failures are server HLS readiness tests, not contract or storage migrations. | If needed, stop and return to planner coordination. |

## Architecture Direction

Keep this as a narrow playback-transcode test-stability lane. Prefer improving
determinism in readiness fixtures, fake runner synchronization, bounded waits,
or test-local resource isolation. Do not widen into artifact I/O pressure,
remote workers, LL-HLS/CMAF, or client behavior.

If investigation proves production HLS runtime behavior is wrong, record the
finding and stop for planner approval before implementing new runtime behavior.

## Closeout Condition

This lane can close when:

- the failing tests pass individually and inside the default full HLS gate;
- `cargo nextest run -p nako-server hls --no-fail-fast` passes fresh;
- `cargo fmt --all -- --check` and `git diff --check` pass;
- HRLB-040 can be retried with updated evidence.

## Closeout Result

This lane closed on 2026-05-31. `HPRTS-020` classified the failure as Windows
full-suite process-backed test timing and added test-only readiness timeout
helpers for the two progressive playlist tests. `HPRTS-030` reran the default
full HLS gate successfully and unblocked HRLB closeout.
