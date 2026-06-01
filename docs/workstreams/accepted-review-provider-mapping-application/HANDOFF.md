# Accepted Review Provider Mapping Application - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is opened from `metadata-candidate-durable-review` closeout. Durable
Metadata Candidate Reviews can be accepted/rejected, but accepted review status
does not currently apply Provider Mapping rows. This lane defines that backend
application boundary before Admin/Web governance depends on it.

## Active Task

- Task ID: `ARPMA-030`
- Owner: codex
- Files: `crates/nako-core`, `crates/nako-metadata`, `crates/nako-db`, and
  this workstream evidence
- Validation: `cargo nextest run -p nako-metadata candidate_review_application --no-fail-fast`;
  `cargo nextest run -p nako-db candidate_review provider_mapping --no-fail-fast`;
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
- `ARPMA-020` shipped the read-only application plan. It is ready to apply new
  root mappings or promote existing candidate mappings, noops existing accepted
  mappings, skips rejected mappings, and exposes unsupported sources as skip
  reasons.

## Blockers

- None for `ARPMA-020`.

## Next Recommended Action

- Run `ARPMA-030`: apply accepted review root Provider Subject and Provider
  Mapping idempotently through existing repository semantics.
