# Provider Hierarchy Application Admin Surface

## Goal

Expose accepted Metadata Candidate Review related hierarchy application through
an explicit Admin API plan/apply surface. The task turns the existing
`nako-metadata` backend capability into an operator-visible workflow without
changing Public Client API, provider endpoint breadth, schema, or Admin Web UI.

## Requirements

- Add a related-hierarchy-specific read-only plan in `nako-metadata`; do not
  reuse the root Provider Mapping application plan as the hierarchy plan.
- Add Admin API DTOs and route inventory entries for related hierarchy
  application plan and apply.
- Add server Admin routes under `/admin/v1/metadata/candidate-reviews/{review_id}`
  that require the existing Admin auth layer and delegate to app services.
- Preserve stale guards with `item_id` and `expected_updated_at_ms`.
- Preserve idempotent replay semantics: a repeated apply after accepted child
  Provider Mappings and non-provisional child states exist returns no change.
- Keep response data redaction-safe: Provider Subject IDs/keys, Provider
  Mapping IDs/status, Media Item IDs, counts, and safe enum reasons are allowed;
  raw provider payloads, source locators, local paths, fingerprints, tokens, and
  provider bodies are not.
- Keep related hierarchy mutation narrow: existing child Media Items only;
  no Media Item creation, parent repair, canonical metadata merge, NFO write,
  Public Client route, Admin Web page, durable batch execution, audit/undo, or
  provider endpoint depth.

## Acceptance Criteria

- [x] Plan endpoint returns a read-only related hierarchy plan for an accepted
      review with an accepted root Provider Mapping.
- [x] Apply endpoint commits safe related Provider Subjects / Provider Mappings
      and marks matched child Library Item State as non-provisional.
- [x] Plan/apply reject pending reviews, missing accepted root mappings,
      ambiguous targets, and unsafe relationship shapes without mutation.
- [x] Admin contract route inventory and generated Admin Web contract are in
      sync.
- [x] Server tests prove Admin-only access, redaction, idempotent replay, and
      no canonical metadata or parent hierarchy mutation.

## Definition of Done

- `cargo fmt --all -- --check`
- `cargo nextest run -p nako-metadata related_hierarchy --no-fail-fast`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo nextest run -p nako-server related_hierarchy --no-fail-fast`
- `cargo check -p nako-core -p nako-metadata -p nako-api -p nako-server --tests`
- `npm run generate:admin-api --prefix apps/admin-web` if generated Admin
  contract output changes.
- `git diff --check`
- Trellis task context validates.

## Technical Approach

Add a related-specific plan in `crates/nako-metadata/src/candidate_review.rs`
that resolves the same safe targets used by `apply_related_hierarchy`, reports
read-only action/reason/count facts, and lets apply call the plan path before
mutation. Map that domain summary into Admin DTOs in `nako-api`, wire route
constants through `admin_contract.rs`, and add thin handlers in
`crates/nako-server/src/http/admin.rs`.

## Decision (ADR-lite)

**Context**: The backend already has a safe apply method, but Admin surfaces need
plan-before-apply. The root Provider Mapping plan reports `noop` once the root
mapping is accepted, so it cannot explain related hierarchy work.

**Decision**: Add a related hierarchy plan type instead of overloading the root
application plan. Keep it Admin-only and synchronous for this narrow operation.

**Consequences**: The plan/apply contract becomes explicit and testable without
schema or durable job work. Bulk related hierarchy execution, Admin Web UX, and
undo/audit remain split follow-ons.

## Out Of Scope

- Admin Web page or action button.
- Public Client API exposure.
- Batch or durable job execution for related hierarchy.
- Audit/undo mutation.
- Media Item creation, reparenting, hierarchy repair, or canonical metadata
  merge.
- Provider endpoint depth or provider-specific matching changes.

## Technical Notes

- Historical backend-only task:
  `.trellis/tasks/archive/2026-06/06-02-01c-provider-review-related-hierarchy-application/prd.md`.
- Current architecture maps keep related hierarchy application split from
  provider governance durable batch execution and Public Client API exposure.
- Relevant code observed:
  `crates/nako-metadata/src/candidate_review.rs`,
  `crates/nako-api/src/admin/metadata_candidate_review.rs`,
  `crates/nako-api/src/admin_contract.rs`,
  `crates/nako-server/src/http/admin.rs`,
  `crates/nako-server/src/http/tests/system.rs`.
