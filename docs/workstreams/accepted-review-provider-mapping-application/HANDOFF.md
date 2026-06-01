# Accepted Review Provider Mapping Application - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is opened from `metadata-candidate-durable-review` closeout. Durable
Metadata Candidate Reviews can be accepted/rejected, but accepted review status
does not currently apply Provider Mapping rows. This lane defines that backend
application boundary before Admin/Web governance depends on it.

## Active Task

- Task ID: `ARPMA-020`
- Owner: codex
- Files: `crates/nako-core/src/media/candidate.rs`,
  `crates/nako-metadata/src/candidate_review.rs`,
  `crates/nako-metadata/src/tests.rs`, and this workstream evidence
- Validation: `cargo nextest run -p nako-metadata candidate_review_application --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`
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

## Blockers

- None for `ARPMA-020`.

## Next Recommended Action

- Run `ARPMA-020`: define the read-only application plan and tests before any
  Provider Mapping mutation service.
