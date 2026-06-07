# M1 RC Closeout Evidence

## Goal

Close the remaining Product-Operator M1 release-candidate evidence gap by
running and recording the RC-relevant docs ladder without skipping the
redaction inventory, then decide whether M1 can be described as RC-ready except
for publication or whether a concrete blocker task must be opened.

## What I Already Know

- `docs/GOALS.md` says roadmap, goal-map, and lane-routing reconciliation are
  complete and that follow-on M1 release-candidate evidence later passed
  `release-fast`, `playback`, `container`, `postgres`, and `workspace`.
- `docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md` says skipped redaction
  inventory is acceptable for local iteration but not for release-candidate
  evidence.
- Existing M1 evidence tasks already record:
  - default/fast ladder evidence,
  - release-fast evidence,
  - playback evidence,
  - container evidence,
  - PostgreSQL evidence,
  - workspace evidence after the HLS timing repair.
- `scripts/m1-release-ladder.ps1 -Mode all` sequences every M1 ladder
  dimension, but it is intentionally expensive and repeats evidence already
  captured in separate archived tasks.
- The clearest missing RC closeout proof is `docs` mode without
  `-SkipRedactionInventory`, plus a single closeout note tying the existing
  separate ladder evidence together.

## Requirements

- Execute:
  `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode docs`
- Do not pass `-SkipRedactionInventory`.
- Record date, git revision, command, host, result, and whether any gate was
  skipped.
- If the docs gate passes, record that the redaction inventory ran and that no
  new M1 blocker implementation task was opened from this closeout.
- If the docs gate fails, classify the failure:
  - formatting or diff hygiene: operations-release docs hygiene;
  - redaction inventory command failure: operations-release tooling;
  - sensitive unredacted output found in committed docs/code: owning feature
    lane by matched surface.
- Keep this task evidence-focused. Do not rerun expensive `all` mode unless the
  docs gate or current evidence contradicts the archived ladder evidence.
- Do not commit raw command logs that may contain local absolute paths. Summarize
  public facts in `evidence.md`.

## Acceptance Criteria

- [x] Trellis context validation passes.
- [x] M1 ladder `docs` mode runs without `-SkipRedactionInventory`.
- [x] `evidence.md` records the command, date, git revision, host, result, and
      skipped-gate state.
- [x] If the gate passes, `evidence.md` ties current separate M1 ladder evidence
      to an RC closeout conclusion.
- [x] If the gate fails, a concrete owner lane and next task route are recorded.
- [x] No Rust, TypeScript, generated contract, schema, or product behavior files
      are changed unless the gate exposes a concrete blocker.

## Definition Of Done

- `evidence.md` contains the closeout result.
- `implement.jsonl` and `check.jsonl` contain real spec context entries, not
  the seeded `_example` rows.
- The task is archived and committed after validation.

## Technical Approach

Treat this as a release evidence task, not an implementation task:

- configure Trellis task context for release/documentation verification;
- run the narrow missing RC gate first;
- summarize sanitized evidence;
- only broaden to implementation work if the gate finds a concrete blocker.

## Decision (ADR-lite)

**Context**: M1 has separate archived evidence for fast, release-fast, playback,
container, PostgreSQL, and workspace modes. The evidence matrix still calls out
that skipped redaction inventory is not acceptable for release candidates.

**Decision**: Run the docs ladder without `-SkipRedactionInventory` and use this
task as the M1 RC closeout evidence binder instead of immediately repeating the
full expensive `all` ladder.

**Consequences**: If docs mode passes, M1 can be treated as RC-ready except for
publication and any intentionally deferred live-browser/package-publication
proof. If it fails, the failure is routed by the evidence matrix instead of
opening speculative M1 work.

## Out Of Scope

- No release artifact publication.
- No crate, package, container, or npm publication.
- No product behavior change unless the gate exposes a blocker.
- No repeated `release-fast`, `playback`, `container`, `postgres`, or
  `workspace` rerun unless current evidence is contradicted.
- No Public Client, Addon Manager, playback breadth, or M2 storage feature work
  inside this task.

## Technical Notes

- Relevant docs:
  - `docs/GOALS.md`
  - `docs/ROADMAP.md`
  - `docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md`
  - `docs/deployment/RELEASE_CHECKLIST.md`
  - `docs/architecture/LANES.md`
  - `docs/architecture/OPERATIONS_RELEASE.md`
- Relevant scripts:
  - `scripts/m1-release-ladder.ps1`
  - `scripts/release-gate.ps1`
- Relevant evidence tasks:
  - `.trellis/tasks/archive/2026-06/06-06-m1-ladder-fast-evidence-run/`
  - `.trellis/tasks/archive/2026-06/06-06-m1-release-fast-evidence-run/`
  - `.trellis/tasks/archive/2026-06/06-06-m1-playback-evidence-run/`
  - `.trellis/tasks/archive/2026-06/06-06-m1-container-evidence-run/`
  - `.trellis/tasks/archive/2026-06/06-06-m1-postgres-evidence-run/`
  - `.trellis/tasks/archive/2026-06/06-06-m1-workspace-evidence-run/`
