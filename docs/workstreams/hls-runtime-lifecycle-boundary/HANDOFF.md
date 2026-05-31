# HLS Runtime Lifecycle Boundary - Handoff

Status: Active
Last updated: 2026-05-31

## Current State

`HRLB-010` is complete as a docs/research invariant freeze. No Rust behavior
changed.

The freeze lives in `DESIGN.md` and covers active same-generation requests,
finished session reuse, different-generation supersede, running playlist
readiness, segment readiness and one-shot wait, cancellation/timeout cleanup,
startup stale-session cleanup, terminal artifact cleanup, staging input
release, and the decision to split artifact I/O pressure into a PAIP follow-on.

Concerns to carry into `HRLB-020`:

- HLS timeout cleanup is implemented in the runner but does not have a focused
  HLS timeout cleanup test.
- Startup stale-session recovery is tested generically for transcode sessions
  but not with an HLS fixture.
- Staging lease primitives are covered, but HLS remote staged-input release is
  not directly covered across success/error/admission-rejection branches.

## Next Task

Planner review should accept or revise the `HRLB-010` freeze before assigning
`HRLB-020`.

Recommended `HRLB-020` focus:

- add focused tests for the frozen invariants above;
- introduce a behavior-preserving lifecycle coordinator/facade only if it
  reduces scattered lifecycle ownership;
- keep `HlsArtifactIo` not-yet-enforced and leave PAIP for a split follow-on.

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

Required validation for `HRLB-020`:

```text
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
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
