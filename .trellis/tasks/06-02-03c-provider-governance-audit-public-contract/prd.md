# Provider Governance Audit And Public Contract

## Goal

Advance provider governance after durable Candidate Review batch execution by
adding a focused trust/visibility slice: audit/undo evidence, Public Client API
governance, or a narrow provider endpoint-depth bridge selected by repo audit.

## Requirements

- Start by auditing the current Candidate Review, Provider Mapping application,
  durable batch, and Admin/Web governance surfaces.
- Choose the smallest slice that improves operator trust or public-client
  readiness without reopening broad provider-depth work.
- Preserve root-only Provider Mapping writes unless the PRD is explicitly
  revised for related hierarchy application.
- Keep raw provider payloads, tokens, headers, proxy URLs, local paths, image
  URLs, source fingerprints, and idempotency keys out of public or Admin
  responses.
- Keep generated Admin/Public contracts synchronized when route shapes change.
- Add focused tests for the selected governance contract.

## Acceptance Criteria

- [x] The worker documents which follow-on is selected and why.
- [x] Existing Candidate Review and durable batch semantics remain compatible.
- [x] New audit/undo/public contract behavior is redaction-safe and testable.
- [x] Generated contracts are updated if API shape changes.
- [x] Follow-ons are split if related hierarchy application or provider endpoint
  depth exceeds the selected scope.

## Definition of Done

- Focused metadata/API/server/db tests pass for changed behavior.
- Web/Admin tests pass if UI changes.
- Generated contract commands run if DTO or route shape changes.
- Evidence notes record selected slice, validation, and deferred follow-ons.

## Out of Scope

- No broad provider-depth reopening.
- No automatic related graph node hierarchy mutation unless planner approves a
  scope revision.
- No raw provider payload exposure.
- No Public Client API route unless the selected slice requires it and tests
  prove the compatibility contract.

## Technical Notes

- Candidate follow-ons from lane docs: `provider-review-public-client-governance`,
  `provider-governance-audit-and-undo`, `douban-tv-episode-endpoint-depth`.
- Likely files: `crates/nako-metadata/src/candidate_review.rs`,
  `crates/nako-server/src/app/metadata_application.rs`,
  `crates/nako-api/src/admin/metadata_candidate_review.rs`,
  `crates/nako-api/src/public_client.rs`, and related DB metadata modules.
- Stop for planner coordination before schema or public contract expansion that
  affects other active lanes.
