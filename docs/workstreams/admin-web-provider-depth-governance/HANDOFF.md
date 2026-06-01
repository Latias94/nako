# Admin Web Provider Depth Governance - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is opened from `accepted-review-provider-mapping-application`
closeout. Backend accepted-review root Provider Mapping application exists, but
Admin API/Web product exposure remains unimplemented.

## Active Task

- Task ID: `AWPDG-020`
- Owner: codex
- Files: `crates/nako-api`, `crates/nako-server`, `crates/nako-metadata`, and
  this workstream evidence
- Validation: `cargo nextest run -p nako-api admin_contract --no-fail-fast`;
  `cargo nextest run -p nako-server candidate_review admin --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`
- Status: READY
- Evidence: `docs/workstreams/admin-web-provider-depth-governance/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Read-only Admin API review/detail/application-plan exposure comes before
  apply mutation.
- Existing Admin Catalog Governance Provider Mapping review routes are
  precedent only; they do not replace durable Candidate Review governance.
- Generated Artifact apply outcome tables remain out of scope.
- Related graph nodes remain preview evidence until a future hierarchy
  application lane.
- Public Client API remains out of scope.

## Blockers

- None for `AWPDG-020`.

## Next Recommended Action

- Run `AWPDG-020`: add a read-only Admin API boundary for durable Candidate
  Review detail and accepted-review application plan evidence.
