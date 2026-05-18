# Core Architecture Deepening Milestones

Status: Completed
Last updated: 2026-05-18

## M0 - Lane Opened

Exit criteria:

- Workstream docs exist and are linked from the workstream index.
- Scope, non-goals, ADR impact, deletion policy, and first executable task are
  explicit.
- `WORKSTREAM.json` named `CAD-020` as the first executable task when the lane
  opened.

## M1 - Durable Commit Units

Exit criteria:

- NFO import uses one production commit interface for its durable write unit.
- Library indexing uses one production commit interface for a discovered Media
  Source write unit.
- Rollback tests prove partial durable state cannot survive failed commits.
- Replaced caller-side write ordering is deleted.

## M2 - Workflow Ports And Locality

Exit criteria:

- Application services touched by this lane no longer need broad `SqliteStore`
  knowledge where a focused workflow port now exists.
- New seams have at least one production adapter and tests that exercise the
  interface rather than SQLite implementation detail where practical.
- Shallow pass-through abstractions introduced during the refactor are removed.

## M3 - Playback/Transcode Identity And Diagnostics

Exit criteria:

- Playback/Transcode Profile identity replaces constant or under-specified HLS
  request keys for persisted session reuse.
- Existing single-variant playback behavior remains compatible.
- Hardware acceleration diagnostics distinguish configured policy, FFmpeg
  encoder evidence, selected fallback, and runtime/smoke capability evidence
  where available.

## M4 - Addon Alignment

Exit criteria:

- Addon Sidecar protected-write follow-ons are checked against the new commit
  interfaces.
- Any addon-specific remaining work is routed to addon workstreams rather than
  duplicated here.

## M5 - Closeout

Exit criteria:

- Focused crate checks and nextest suites have fresh evidence.
- Workspace check, workspace nextest, formatting, and diff checks are recorded
  or explicitly blocked with reason.
- `WORKSTREAM.json`, `HANDOFF.md`, and this milestone file reflect the final
  state.

Outcome:

- All milestone exit criteria are met for the scoped workstream.
- Workspace closeout gates passed on 2026-05-18:
  `cargo check --workspace --tests`, `cargo nextest run --workspace --no-fail-fast`,
  `cargo fmt --all -- --check`, and `git diff --check`.
- Remaining artwork, subtitle, NFO export, and Library File Write product
  breadth stays in the dedicated addon follow-on workstreams rather than this
  closed lane.
