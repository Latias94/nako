# Accepted Review Provider Mapping Application - Handoff

Status: Closed
Last updated: 2026-06-02

## Current State

The lane is closed after proving accepted-review root Provider Subject /
Provider Mapping application as an explicit backend service. Admin/API/Web
exposure is split to `proposed:admin-web-provider-depth-governance`.

## Active Task

- Task ID: none
- Owner: none
- Files: none
- Validation: see `CLOSEOUT.md`
- Status: DONE
- Evidence: `docs/workstreams/accepted-review-provider-mapping-application/CLOSEOUT.md`

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
- `ARPMA-040` split Admin API/Web mutation scope to
  `proposed:admin-web-provider-depth-governance`.
- `ARPMA-050` closed this backend lane.

## Blockers

- None for this closed lane.

## Next Recommended Action

- Open `proposed:admin-web-provider-depth-governance` before exposing durable
  Metadata Candidate Review evidence or accepted-review application mutations
  through Admin API/Web.
