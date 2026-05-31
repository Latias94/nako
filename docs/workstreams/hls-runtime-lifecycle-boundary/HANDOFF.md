# HLS Runtime Lifecycle Boundary - Handoff

Status: Closed
Last updated: 2026-05-31

## Current State

This workstream is closed.

`HRLB-010` froze HLS lifecycle invariants for active/reuse/supersede,
readiness, one-shot segment wait, cancellation, timeout cleanup, startup stale
recovery, terminal artifact cleanup, staging input release, and the explicit
decision to keep artifact I/O pressure as a separate PAIP follow-on.

`HRLB-020` added behavior-preserving HLS lifecycle coverage for timeout
cleanup, HLS stale startup recovery, and remote staged-input release across
success, runner error, and admission rejection. No lifecycle coordinator or
facade was introduced because the useful slice was coverage and evidence.

`HRLB-030` split non-lifecycle expansion work into follow-ons:

- `hls-progressive-readiness-test-stability`
- `proposed:hls-artifact-io-pressure-enforcement`
- `proposed:playback-admission-queueing-and-waitlist`
- `proposed:remote-transcode-worker-runtime`
- `proposed:ll-hls-cmaf-runtime`
- `proposed:player-hls-session-controls-and-recovery`

`HRLB-040` first found that the required full HLS gate failed under default
nextest concurrency on two progressive readiness tests. That instability was
split to `hls-progressive-readiness-test-stability`. After `HPRTS-020` fixed
the test-only Windows readiness timeout and `HPRTS-030` verified the full HLS
gate, HRLB closeout was retried and completed.

## Completed

- Task ID: `HRLB-010`, `HRLB-020`, `HRLB-030`, `HRLB-040`
- Lane: `playback-transcode`
- Final status: DONE_WITH_CONCERNS

## Validation

Fresh planner verification on 2026-05-31:

```text
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json
python -m json.tool docs/workstreams/hls-progressive-readiness-test-stability/WORKSTREAM.json
git diff --check
```

Result: passed. The full HLS gate ran 71 tests, all passed, with 26 slow tests.

## Residual Risks And Follow-Ons

- PAIP artifact I/O pressure remains split and must coordinate with
  storage/VFS health and playback resource demand.
- Resource admission queueing/waitlists remain a separate playback scheduler
  follow-on.
- Remote transcode workers remain a control-plane/runtime follow-on.
- LL-HLS/CMAF and player session controls remain separate protocol/client
  follow-ons.
- Broader Windows HLS fixture scheduling or nextest grouping may be needed if
  the process-backed HLS suite keeps growing slower.

## Next Recommended Action

Open a dedicated follow-on before starting PAIP, queueing, remote workers,
LL-HLS/CMAF, or player UX. Do not reopen HRLB for those scopes.
