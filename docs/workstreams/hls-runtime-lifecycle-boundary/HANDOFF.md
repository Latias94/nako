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

## HRLB-030 Decisions

`HRLB-030` is complete with concerns. Follow-on decisions:

- next recommended bounded workstream:
  `proposed:hls-progressive-readiness-test-stability`;
- PAIP artifact I/O pressure remains separate:
  `proposed:hls-artifact-io-pressure-enforcement`;
- resource admission unification remains separate:
  `proposed:playback-admission-queueing-and-waitlist`;
- remote workers remain separate:
  `proposed:remote-transcode-worker-runtime`;
- LL-HLS/CMAF remains separate:
  `proposed:ll-hls-cmaf-runtime`;
- player UX remains separate:
  `proposed:player-hls-session-controls-and-recovery`.

The reason to prioritize HLS test stability is HRLB-020's load-sensitive
progressive-readiness evidence. PAIP should wait until the HLS gate is stable
because it will add read/write pressure and more concurrent segment behavior.

## Next Task

Assign `HRLB-040` for closeout. Verify final gates, preserve HRLB-030's
follow-on decisions, and close this lifecycle boundary workstream or split any
remaining closeout-only follow-ons.

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

Required validation for `HRLB-040`:

```text
final gates from EVIDENCE_AND_GATES.md
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
