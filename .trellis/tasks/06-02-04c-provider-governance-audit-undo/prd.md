# Provider Governance Audit Undo

## Goal

Increase operator trust in provider governance by adding a bounded audit and
undo slice for Candidate Review / Provider Mapping application without exposing
raw provider payloads or reopening broad provider-depth work.

## Requirements

- Audit current Candidate Review detail/apply, durable batch execution, Provider
  Mapping writes, Admin/Web governance surfaces, and existing outcome evidence.
- Select the smallest audit/undo slice that improves trust. A read-only undo
  plan or replay-safe audit timeline is acceptable if full mutation undo would
  exceed safe scope.
- Preserve root-only Provider Mapping semantics unless planner explicitly
  approves related hierarchy application.
- Keep raw provider payloads, tokens, headers, proxy URLs, local paths, image
  URLs, Source Fingerprints, idempotency keys, and raw errors out of Admin and
  Public responses.
- Keep Public Client API governance unchanged unless a separate public contract
  decision is approved.
- Add focused tests for audit/undo safety, idempotency, redaction, and stale
  state behavior.

## Acceptance Criteria

- [ ] The worker documents the selected audit/undo slice and why broader undo is
  deferred if needed.
- [ ] Existing Candidate Review and durable batch behavior remains compatible.
- [ ] Audit or undo output is redaction-safe and testable.
- [ ] Undo mutation, if implemented, is stale-state safe and replay-safe.
- [ ] Generated Admin/Web contracts are updated if API shape changes.
- [ ] Follow-ons are split for related hierarchy application or provider
  endpoint-depth work.

## Definition of Done

- Focused metadata/API/server/db tests pass for changed behavior.
- Web/Admin tests pass if UI changes.
- Generated contract commands/tests run if DTO or route shape changes.
- Evidence notes record selected slice, validation, and deferred follow-ons.

## Out of Scope

- No broad provider endpoint-depth reopening.
- No automatic related graph node hierarchy mutation unless planner approves a
  scope revision.
- No raw provider payload exposure.
- No Public Client API route unless the task is explicitly revised.

## Technical Notes

- Likely files: `crates/nako-metadata/src/candidate_review.rs`,
  `crates/nako-server/src/app/metadata_application.rs`,
  `crates/nako-api/src/admin/metadata_candidate_review.rs`,
  DB metadata modules, and Admin/Web governance tests.
- Stop for planner coordination before schema or public contract expansion.
