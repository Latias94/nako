# HLS Runtime Lifecycle Boundary - Handoff

Status: Active
Last updated: 2026-05-31

## Current State

`HRLB-010` is complete as a docs/research invariant freeze.

The freeze lives in `DESIGN.md` and covers active same-generation requests,
finished session reuse, different-generation supersede, running playlist
readiness, segment readiness and one-shot wait, cancellation/timeout cleanup,
startup stale-session cleanup, terminal artifact cleanup, staging input
release, and the decision to split artifact I/O pressure into a PAIP follow-on.

`HRLB-020` is complete with concerns. It added behavior-preserving focused
coverage for:

- HLS timeout failure mapping, persisted `Timeout` category, and serve-visible
  output cleanup;
- HLS-specific startup stale-session recovery;
- HLS remote staged-input release after success, runner error, and admission
  rejection.

No lifecycle coordinator/facade was introduced. HRLB-010 justified focused
tests first, and this patch did not uncover a behavior-preserving abstraction
that clearly reduces lifecycle ownership drift.

Planner accepted a narrow scope-out compile fix for Admin hardware diagnostics:
`HardwarePipelineStage::{ToneMap, SubtitleBurnIn}` now map to
`AdminHardwarePipelineStage`.

Validation:

```text
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Result: passed on 2026-05-31. An earlier full HLS run had one existing
load-sensitive progressive-readiness timeout; that test passed individually and
the final full rerun passed 70/70.

## Next Task

Assign `HRLB-030` for planner follow-on split decisions. Decide whether PAIP
artifact I/O pressure, resource admission unification, remote workers,
LL-HLS/CMAF, player UX, or HLS test stability become separate workstreams.

Required context:

```text
docs/workstreams/hls-runtime-lifecycle-boundary/CONTEXT.jsonl
docs/adr/0052-hls-runtime-and-media-engine-boundary.md
docs/architecture/PLAYBACK.md
docs/architecture/LANES.md
docs/workstreams/transcode-capability-inventory-matrix/CLOSEOUT.md
docs/workstreams/hdr-tone-mapping-pipeline/CLOSEOUT.md
docs/workstreams/hls-progressive-runtime-boundary/HANDOFF.md
docs/workstreams/hls-seek-restart-lifecycle/HANDOFF.md
docs/workstreams/playback-runtime-resource-scheduler/HANDOFF.md
docs/workstreams/remote-storage-health-and-circuit-breaker/CLOSEOUT.md
```

Required validation for `HRLB-030`:

```text
python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json
git diff --check
```

## Stop Conditions

Return to planner coordination if:

- implementation needs `nako-transcode` pipeline selection or FFmpeg command
  planning;
- the task needs Public/Admin DTO changes or storage schema changes;
- artifact I/O pressure requires storage health/circuit-breaker behavior
  changes;
- client/player UX, LL-HLS/CMAF, DASH/CMAF, DRM/key delivery, or remote worker
  execution becomes necessary.

## Report Format

End with one of:

- DONE
- DONE_WITH_CONCERNS
- BLOCKED
- NEEDS_CONTEXT

Include files changed, invariant coverage, validation evidence, and follow-ons
split.
