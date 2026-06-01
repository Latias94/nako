# Accepted Review Provider Mapping Application - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is opened from `metadata-candidate-durable-review` closeout. Durable
Metadata Candidate Reviews can be accepted/rejected, and accepted review root
Provider Subject / Provider Mapping application now exists as an explicit
backend service rather than a hidden status-transition side effect. Admin/Web
governance remains unexposed until `ARPMA-040` decides the surface split.

## Active Task

- Task ID: `ARPMA-040`
- Owner: planner
- Files: this workstream evidence, architecture maps, `docs/GOALS.md`, and
  `docs/ROADMAP.md`
- Validation: fresh evidence in `EVIDENCE_AND_GATES.md`; JSON/JSONL
  validation; `git diff --check`
- Status: READY
- Evidence: `docs/workstreams/accepted-review-provider-mapping-application/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Plan before apply.
- Root Provider Subject / Provider Mapping only in this lane.
- Unsupported review sources must be explicit plan reasons.
- Related graph nodes remain preview evidence for future Admin/Web or hierarchy
  governance.
- Do not reuse Generated Artifact apply outcome tables as candidate review
  application state.
- Do not create a second Generated Artifact metadata apply executor.
- `ARPMA-020` shipped the read-only application plan. It is ready to apply new
  root mappings or promote existing candidate mappings, noops existing accepted
  mappings, skips rejected mappings, and exposes unsupported sources as skip
  reasons.
- `ARPMA-030` shipped `MetadataCandidateReviewApplicationService`, which applies
  only the root Provider Subject / Provider Mapping idempotently, rejects
  rejected mapping conflicts, and leaves related review graph nodes as preview
  evidence.

## Blockers

- None for `ARPMA-040`.

## Next Recommended Action

- Run `ARPMA-040`: decide whether Admin API/Web mutation scope belongs in this
  lane or should split with Admin/Web provider depth governance before exposing
  the backend application service.
