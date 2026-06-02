# Admin Candidate Review List Navigation - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is opened from `admin-web-provider-depth-governance` closeout. Durable
Candidate Review detail/apply exists, but Web/Admin discovery still requires a
known `review_id`. The existing repository seam can list Candidate Reviews for
one Media Item, so the first executable task is a read-only Admin API list
surface.

## Active Task

- Task ID: `ACRN-020`
- Owner: codex
- Files: `crates/nako-api`, `crates/nako-server`, `crates/nako-metadata`, and
  `docs/workstreams/admin-candidate-review-list-navigation`
- Validation: `cargo nextest run -p nako-api admin_contract --no-fail-fast`;
  `cargo nextest run -p nako-server candidate_review admin --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`
- Status: READY
- Evidence: `docs/workstreams/admin-candidate-review-list-navigation/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Start item-scoped before global queue/search.
- List rows are summaries for triage/navigation, not full Candidate Review
  detail duplication.
- ACRN-020 is read-only and must not write Provider Subject, Provider Mapping,
  Canonical Metadata, or related graph hierarchy state.
- Web navigation waits for ACRN-030 and should route into the existing
  detail/apply page instead of adding another apply path.

## Blockers

- None for `ACRN-020`.

## Next Recommended Action

- Run `ACRN-020`: add the Admin API item-scoped Candidate Review list route,
  DTOs, route inventory/generated contract sync, and redaction/no-write tests.
