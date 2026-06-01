# MVP Release Shape - Milestones

Status: Active
Last updated: 2026-06-01

## M0 - Scope And Evidence Freeze

Outcome: the MVP release convergence lane exists and has an initial release
cut.

Exit criteria:

- `WORKSTREAM.json` is valid.
- MVP statement and non-goals are recorded.
- Architecture links point to this workstream.
- Active queue implications are noted.

## M1 - Release Cut Verification

Outcome: every P0/P1/P2 claim is verified against repository evidence.

Exit criteria:

- `GAP_MATRIX.md` identifies blocker, non-blocker, and deferred rows.
- Active workstreams are classified as MVP-blocking or post-MVP.
- Related repo requirements are explicit and not assumed.

## M2 - MVP Gate Plan

Outcome: the first release can be validated as a complete user journey.

Exit criteria:

- Install/startup gate is named.
- Scan/metadata/playback/Admin/addon/network gates are named.
- Redaction and diagnostics gates are named.
- Missing gates are routed to workstreams.

## M3 - Active Queue Alignment

Outcome: current active tails no longer obscure MVP planning.

Exit criteria:

- `PTJCH` is finished, split, or scoped as an MVP blocker.
- `GAMA` is finished, split, or explicitly deferred from MVP.
- `CSAPA` has desktop playback split or deferral recorded.

## M4 - Closeout Or Campaign Split

Outcome: the planner can assign implementation campaigns without re-litigating
MVP scope.

Exit criteria:

- MVP blocker campaigns have exact owners, worktrees, gates, and stop
  conditions.
- Post-MVP work is not in the active release path.
- The workstream is closed or left active with one current task.
