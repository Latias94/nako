# Admin Source Duplicate Reconciliation Plan Read-Only API

## Goal

Expose the internal read-only Source Duplicate reconciliation plan through a
versioned Admin API route so operators can inspect same-library duplicate
suggestions from redacted Source Fingerprint evidence without applying any
catalog mutation.

## Requirements

- Add Admin DTOs for a source duplicate reconciliation plan response.
- Add a versioned Admin route:
  `GET /admin/v1/libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-plan`.
- Accept bounded pagination through existing Admin pagination query conventions.
- Require Admin authorization using the existing Admin route guard.
- Delegate to
  `SourceDuplicateReconciliationAppService::plan_source_duplicate_reconciliation`.
- Return only redaction-safe fields:
  - `library_id`
  - `source_id`
  - `fingerprint_evidence_kind`
  - `confidence_milli`
  - `stale`
  - candidate `source_id`
  - candidate `duplicate_source_id`
  - candidate `evidence_kind`
  - candidate `confidence_milli`
  - candidate `stale`
  - candidate `relationship_id`
  - candidate `existing_status`
  - candidate `recommended_action`
- Preserve existing Suggested, Confirmed, and Rejected relationship status in
  the DTO.
- Keep the route read-only: no Source Duplicate Relationship write, no Media
  Source merge, no Media Item merge, no Playback Source Selection mutation, and
  no Library Access mutation.
- Keep errors and responses redaction-safe. Do not expose Source Locators, local
  paths, etags, backend URLs, credentials, raw hashes, raw fingerprints,
  evidence values, durable job input JSON, or source fingerprint material.
- Register the route in Admin route inventory / generated Admin TypeScript
  contract output if route inventory generation owns that surface.

## Acceptance Criteria

- [ ] Admin can request a duplicate reconciliation plan for a library/source.
- [ ] Non-admin callers are rejected.
- [ ] The route rejects or errors safely for missing source, cross-library
      source, missing fingerprint evidence, and unsupported raw fingerprint
      evidence.
- [ ] Same-library duplicate candidates are returned with existing relationship
      status and recommended action.
- [ ] Pagination is candidate-oriented: the target source is excluded before
      `limit`/`offset`.
- [ ] The response and error body do not contain Source Locators, local paths,
      raw fingerprints, raw hashes, etags, backend URLs, credentials, or
      durable job input JSON.
- [ ] The route does not mutate duplicate relationships.
- [ ] Admin contract generation/tests cover the new route and DTOs.

## Definition Of Done

- Admin DTOs compile and serialize with snake_case fields.
- Server Admin route delegates to the app service and maps the plan through an
  explicit DTO conversion.
- Focused HTTP tests cover success, auth guard, redaction, unsafe input
  failures, pagination, and read-only behavior.
- Admin API contract tests pass when route inventory or generated DTO output
  changes.
- `cargo check -p nako-api -p nako-server --tests` passes.
- Focused `cargo nextest` gates pass.
- `cargo fmt --all -- --check`, `git diff --check`, and Trellis context
  validation pass.

## Out Of Scope

- No reconciliation apply endpoint.
- No automatic source hash completion writer.
- No automatic scan-originated reconciliation scheduling.
- No auto-confirmed duplicate relationship.
- No Media Source or Media Item merge.
- No Public Client API route.
- No Admin Web UI.
- No schema migration.
- No raw fingerprint/evidence diagnostics route.

## Technical Approach

- Add Admin DTOs under the existing `nako-api` Admin operations/source hash
  surface or a nearby Admin governance module.
- Add an Admin HTTP handler in `nako-server` that:
  - extracts `library_id`, `source_id`, and pagination query;
  - requires the existing Admin principal guard;
  - calls `app.source_duplicate_reconciliation()`;
  - maps the app plan into the Admin DTO.
- Register the route in the Admin router and route inventory.
- Add focused route tests using existing Admin auth helpers and media-source
  fixtures.
- Regenerate Admin contract output if the repo's contract test requires it.

## Decision (ADR-lite)

**Context**: The internal source duplicate reconciliation plan is already
read-only and redacted. Operators need a safe Admin surface before any future
apply/reopen workflow can be designed.

**Decision**: Ship a read-only Admin plan route first. Keep apply, automatic
mutation, and Web UI as explicit follow-ons.

**Consequences**: Admin and UI work can inspect reconciliation pressure without
surprising catalog mutation. The later apply workflow can reuse the same plan
shape but still needs a separate idempotent command, confirmation semantics, and
undo/audit decision.

## Technical Notes

- Predecessor implementation:
  `.trellis/tasks/archive/2026-06/06-06-source-duplicate-reconciliation-plan-first-slice/`
- Source hash / duplicate reconciliation server spec:
  `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`
- Admin contract spec:
  `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
