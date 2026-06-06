# M1 Ladder Evidence Matrix

## Goal

Add a release-facing evidence matrix for the Product-Operator M1 ladder. The
matrix should connect each ladder mode to the M1 journey evidence it proves,
the command to run, required tooling, current evidence state, and the follow-on
route when a gate fails or is skipped.

## What I Already Know

- `scripts/m1-release-ladder.ps1` shipped and supports `docs`, `smoke`,
  `fast`, `release-fast`, `playback`, `container`, `postgres`, `workspace`,
  and `all`.
- Default `fast` mode delegates to the docs-safe release gate and the focused
  M1 operator journey smoke.
- The release ladder runner task proved `docs` and `fast` locally.
- Expensive release dimensions are intentionally explicit: playback,
  container, PostgreSQL, workspace, and all-mode validation.
- `docs/deployment/RELEASE_CHECKLIST.md` explains how to run the ladder, but it
  does not yet show the release-readiness evidence matrix or skip rules.

## Requirements

- Add `docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md`.
- The matrix must cover every `scripts/m1-release-ladder.ps1` mode.
- The matrix must map M1 journey areas to concrete runner modes and current
  evidence sources.
- The matrix must state when skipped evidence is acceptable and how to record
  skipped environment-dependent gates.
- Update `docs/deployment/RELEASE_CHECKLIST.md` to point operators at the
  matrix.
- Update `docs/architecture/OPERATIONS_RELEASE.md` so operations/release
  architecture records the matrix as shipped evidence.
- Update Trellis/spec guidance so future runner mode changes also update the
  evidence matrix.
- Do not change Rust, TypeScript, generated contracts, runtime behavior, or
  runner scripts in this slice.

## Success Metrics

| Metric | Target | Measurement |
| --- | --- | --- |
| Mode coverage | 9/9 ladder modes documented | Compare matrix against `ValidateSet` in `scripts/m1-release-ladder.ps1` |
| Evidence routing | Each M1 quality-gate area has a command and follow-on route | Manual review of the matrix |
| Operator clarity | Environment-dependent gates have explicit skip recording rules | Manual review |
| Scope containment | Only deployment/operations/spec docs and this task evidence change | `git status --short` and staged diff |

## Alternatives Considered

### Option A: Add A Dedicated Evidence Matrix Document

Pros:

- Keeps the release checklist short while giving release candidates a durable
  evidence ledger.
- Can be updated without touching runner logic.
- Makes skipped environment-dependent checks explicit.

Cons:

- Adds one more release document to maintain.

Decision: chosen because this task is about evidence clarity, not script
behavior.

### Option B: Put The Whole Matrix Inside `RELEASE_CHECKLIST.md`

Pros:

- One document for operators to open.

Cons:

- The checklist is already a procedural release guide.
- A large matrix would make the normal install/release checklist harder to
  scan.

Decision: rejected; the checklist should link to the detailed matrix.

### Option C: Generate The Matrix From The Runner Script

Pros:

- Reduces future drift risk if implemented well.

Cons:

- Requires new tooling and a structured metadata format in the script.
- Overkill before the matrix has stabilized through one release candidate.

Decision: rejected for this slice; record a spec rule that runner mode changes
must update the matrix.

## Risks And Mitigations

| Risk | Severity | Likelihood | Mitigation |
| --- | --- | --- | --- |
| Matrix drifts from runner modes | Medium | Medium | Spec requires future runner mode changes to update the matrix and validate mode coverage |
| Operators treat optional gates as passed | High | Medium | Matrix requires skipped gates to be recorded with reason, environment, and follow-up owner |
| Evidence document becomes too broad | Medium | Low | Keep matrix tied to M1 ladder modes and release-quality areas only |
| Docs-only change misses a script mismatch | Medium | Low | Validate mode names with `rg` and run docs-safe ladder mode |

## Acceptance Criteria

- [x] `docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md` exists and covers all
      ladder modes.
- [x] Release checklist links to the matrix from section 5a.
- [x] Operations release architecture records the matrix as shipped evidence.
- [x] Trellis/spec guidance states that runner mode changes must update the
      matrix.
- [x] Trellis context validation passes.
- [x] Docs-safe ladder validation passes.
- [x] `git diff --check` passes for touched docs and task evidence.

## Definition Of Done

- Matrix and cross-links are written.
- Validation evidence is recorded.
- Trellis task is archived.
- Work is committed and pushed.

## Verification Evidence

- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-m1-ladder-evidence-matrix`
  passed with 6 implement context entries and 6 check context entries.
- `rg -n "docs|smoke|fast|release-fast|playback|container|postgres|workspace|all" scripts/m1-release-ladder.ps1 docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md`
  confirmed the matrix covers every runner mode from
  `scripts/m1-release-ladder.ps1`.
- `git diff --check -- .trellis/tasks/06-06-m1-ladder-evidence-matrix docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md docs/deployment/RELEASE_CHECKLIST.md docs/architecture/OPERATIONS_RELEASE.md .trellis/spec/nako-server/backend/quality-guidelines.md`
  passed for tracked changes. Git reported LF-to-CRLF working-copy warnings
  for existing markdown files, but no whitespace errors.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode docs -SkipRedactionInventory`
  passed. Nested `release-gate.ps1 -Mode docs -SkipRedactionInventory` ran
  `cargo fmt --all -- --check` and `git diff --check`.
- Rust, TypeScript, browser, playback, container, PostgreSQL, and workspace
  gates beyond docs-safe mode were not run because this slice changes only
  release documentation and Trellis task evidence.
