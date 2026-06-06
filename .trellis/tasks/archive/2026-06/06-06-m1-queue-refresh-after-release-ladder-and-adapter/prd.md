# M1 Queue Refresh After Release Ladder And Adapter

## Goal

Refresh the M1 release convergence queue after the release ladder runner and
Admin Web feature data adapter slices shipped. The roadmap, goal map, and lane
routing should stop naming completed tasks as next work and should point the
next execution step at evidence-driven M1 readiness triage.

## What I Already Know

- `m1-release-ladder-runner` shipped through
  `.trellis/tasks/archive/2026-06/06-06-m1-release-ladder-runner/`.
- `admin-web-feature-data-adapter-deepening` shipped through
  `.trellis/tasks/archive/2026-06/06-06-admin-web-feature-data-adapter-deepening/`.
- The release ladder fast mode passed after composing docs-safe hygiene and the
  focused M1 operator journey smoke.
- The Admin Web adapter slice was an architecture locality follow-on, not a
  player or backend release blocker.
- `docs/ROADMAP.md`, `docs/GOALS.md`, and `docs/architecture/LANES.md` still
  name the shipped tasks as next candidates.

## Requirements

- Move `m1-release-ladder-runner` and
  `admin-web-feature-data-adapter-deepening` into completed M1 convergence
  evidence.
- Replace the stale next queue with post-runner candidates:
  - `m1-ladder-evidence-matrix`;
  - `media-web-library-browse-and-player-smoke` only when ladder evidence
    exposes a concrete browser/player blocker;
  - `m1-admin-diagnostics-repair-coverage-audit` for Admin diagnostics/repair
    readiness gaps.
- Keep the queue Product-Operator M1 oriented: install/config, scan, catalog,
  playback, Admin diagnostics/repair, redaction, packaging/container, and
  focused Web/API smoke.
- Do not reopen archived Trellis tasks or legacy workstreams.
- Do not change Rust, TypeScript, schema, generated contracts, runtime
  behavior, or release scripts in this slice.

## Evidence Flow

```text
Archived M1 slices
  -> completed evidence tables
  -> post-runner queue
  -> evidence matrix task
  -> concrete blocker task only when a gate fails or exposes a gap
```

## Acceptance Criteria

- [x] `docs/ROADMAP.md` lists the runner and adapter slices as completed
      evidence.
- [x] `docs/ROADMAP.md` no longer presents the completed runner or adapter
      tasks as next executable work.
- [x] `docs/GOALS.md` records the post-runner queue and evidence-driven next
      step.
- [x] `docs/architecture/LANES.md` routes operations-release to the evidence
      matrix and keeps Media Web/player work conditional on real blocker
      evidence.
- [x] Trellis context validation passes.
- [x] `git diff --check` passes for the task and edited docs.

## Success Metrics

| Metric | Target | Measurement |
| --- | --- | --- |
| Completed task duplication | 0 stale completed tasks in next queue | Manual review of roadmap, goal map, and lanes |
| Evidence links | Runner and adapter archive paths present | Manual review |
| Next task clarity | At least one concrete immediate next task | Manual review |
| Scope containment | Only planning docs and this Trellis task change | `git status --short` and staged diff |

## Alternatives Considered

### Option A: Refresh The Queue Now

Pros:

- Keeps planning authority aligned with committed evidence.
- Prevents the next agent from reopening completed work.
- Creates a clean handoff into evidence-driven release readiness triage.

Cons:

- Adds another docs-only slice before more implementation.

Decision: chosen because the current queue is already stale after two shipped
M1 slices.

### Option B: Skip Queue Refresh And Start Media Web Work

Pros:

- Moves directly into implementation.

Cons:

- The fast release ladder did not expose a Media Web blocker.
- This would violate the current queue rule that Media Web/player work should
  be blocker-driven.

Decision: rejected until evidence shows a concrete browse/player failure.

### Option C: Broadly Redesign M1 Planning

Pros:

- Could produce a deeper release program.

Cons:

- Too broad for the current goal and risks hiding the simple stale-queue fix.
- Would create churn in historical roadmap sections without new evidence.

Decision: rejected for this slice.

## Risks And Mitigations

| Risk | Severity | Likelihood | Mitigation |
| --- | --- | --- | --- |
| Queue becomes too docs-heavy | Medium | Medium | Name `m1-ladder-evidence-matrix` as an execution/evidence task, not another broad planning pass |
| Media Web blocker is missed | Medium | Low | Keep Media Web task conditional but explicitly listed for ladder/player failures |
| Historical evidence is obscured | Medium | Low | Move completed work into evidence tables and keep archive paths visible |
| Scope creep into implementation | Low | Medium | Limit file scope to roadmap, goals, lanes, and task evidence |

## Definition Of Done

- Planning docs reflect the post-runner/post-adapter M1 queue.
- Context files are curated and validated.
- Validation evidence is recorded.
- Work is committed and pushed.
- Trellis task is archived.

## Verification Evidence

- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-m1-queue-refresh-after-release-ladder-and-adapter`
  passed with 5 implement context entries and 5 check context entries.
- `rg -n "m1-release-ladder-runner|admin-web-feature-data-adapter-deepening|m1-ladder-evidence-matrix|m1-admin-diagnostics-repair-coverage-audit|Next executable queue" docs/ROADMAP.md docs/GOALS.md docs/architecture/LANES.md`
  confirmed the completed runner and adapter appear as evidence while the next
  queue points at `m1-ladder-evidence-matrix`.
- `git diff --check -- docs/ROADMAP.md docs/GOALS.md docs/architecture/LANES.md .trellis/tasks/06-06-m1-queue-refresh-after-release-ladder-and-adapter`
  passed. Git reported LF-to-CRLF working-copy warnings for existing markdown
  files, but no whitespace errors.
- Rust, TypeScript, and browser tests were not run because this slice changes
  only planning docs and Trellis task evidence.
