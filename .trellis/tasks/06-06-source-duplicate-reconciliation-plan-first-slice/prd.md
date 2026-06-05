# Source Duplicate Reconciliation Read-Only Plan First Slice

## Goal

Add an internal, read-only source duplicate reconciliation planning service that
uses persisted redacted Source Fingerprint evidence to identify same-library
duplicate candidates and recommend safe next actions without mutating duplicate
relationships.

## Requirements

- Add core domain types for a `SourceDuplicateReconciliationPlan` and candidate
  actions.
- Recover Source Fingerprint evidence kind and confidence only from the persisted
  redacted fingerprint format:
  `source:v1:<kind>:sha256:<digest>`.
- Add repository reads needed by the planner:
  - list same-library Media Sources by persisted fingerprint with bounded
    candidate-oriented pagination, target-source exclusion, and stale Source
    State projection;
  - look up an existing Source Duplicate Relationship by canonical source pair.
- Add a server app service method:
  `SourceDuplicateReconciliationAppService::plan_source_duplicate_reconciliation`.
- Validate that the requested source exists, belongs to the requested Media
  Library, and has supported redacted fingerprint evidence.
- Return source ids, evidence kind, confidence, stale flag, existing relationship
  id/status, and a recommended action.
- Preserve existing Suggested, Confirmed, and Rejected relationship statuses in
  the plan.
- Recommend fingerprint refresh when the source or candidate evidence is stale.
- Recommend only a Suggested relationship for fresh unmatched candidates.
- Keep the plan read-only: no `SourceDuplicateRelationship` writes, no Media
  Source merge, no Media Item merge, no Playback Source Selection change, and no
  Library Access mutation.
- Keep plan and error surfaces redaction-safe. Do not expose Source Locators,
  local paths, etags, backend URLs, credentials, raw hash material, raw
  fingerprints, evidence values, or durable job input JSON.

## Acceptance Criteria

- [ ] A source with a content hash produces candidates for same-library sources
      sharing the same redacted fingerprint.
- [ ] Cross-library sources with the same fingerprint are excluded.
- [ ] The requested source is excluded before candidate pagination is applied.
- [ ] Existing Suggested, Confirmed, and Rejected relationships are preserved in
      candidate actions.
- [ ] Stale source state is reflected in candidate stale/confidence fields and
      recommends refresh when no existing relationship exists.
- [ ] The planner does not write duplicate relationships.
- [ ] Missing, cross-library, missing-fingerprint, and unsupported raw
      fingerprint inputs fail without leaking unsafe source details.
- [ ] SQLite and PostgreSQL repository contracts cover fingerprint match reads,
      pagination, stale projection, and canonical pair lookup.
- [ ] Focused server and DB tests pass.

## Definition Of Done

- Core domain/repository contracts compile.
- SQLite and PostgreSQL adapters implement the new repository reads.
- App-service tests cover read-only behavior, redaction, existing status
  preservation, stale evidence handling, and unsafe input rejection.
- DB contract tests cover same-library bounded fingerprint reads and pair lookup.
- Relevant Trellis specs capture the durable read-only reconciliation contract.
- `cargo check` and focused `cargo nextest` gates pass.
- `cargo fmt --all -- --check` and `git diff --check` pass.

## Out Of Scope

- No Admin or Public Client route.
- No reconciliation apply endpoint.
- No automatic source hash completion writer.
- No automatic scan-originated reconciliation scheduling.
- No auto-confirmed duplicate relationship.
- No Media Source or Media Item merge.
- No schema migration.
- No raw fingerprint/evidence diagnostics route.

## Technical Approach

- Add redacted fingerprint parsing helpers to `nako-core` Source Fingerprint
  domain types.
- Add reconciliation plan/candidate/action records in `nako-core`.
- Extend `MediaRepository` with a bounded same-library fingerprint match read.
- Extend `SourceDuplicateRepository` with canonical pair lookup.
- Implement SQLite and PostgreSQL adapter parity for the new repository reads.
- Add a server app service under `nako-server::app` that orchestrates only
  repository reads and redacted domain mapping.
- Keep the first slice internal to the app/service boundary so a later Admin
  route can be added with an explicit API contract and pagination DTO.

## Decision (ADR-lite)

**Context**: Source hash execution now persists redacted Source Fingerprint
evidence, and duplicate relationship pair upsert is idempotent. The next safe
step is to show what Nako would suggest before enabling any writer.

**Decision**: Add a read-only app-service plan first. The plan may inspect
fingerprint evidence and existing relationships, but it must not create or
update duplicate relationships.

**Consequences**: Future Admin/UI work can expose the same planning contract
without coupling hash execution to catalog mutation. Apply/undo/audit behavior
remains a separate product decision.

## Technical Notes

- Parent wave:
  `.trellis/tasks/06-06-06-06-fearless-refactor-development-wave/`
- Predecessor pair-idempotency task:
  `.trellis/tasks/06-06-source-duplicate-relationship-idempotent-pair-upsert/`
- Source hash policy spec:
  `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`
