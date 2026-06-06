# Admin Source Duplicate Reconciliation Apply First Slice

## Goal

Add a narrow Admin-only apply command for source duplicate reconciliation plans
so an operator can explicitly materialize one fresh `suggest_relationship`
candidate as a Suggested Source Duplicate Relationship.

## Requirements

- Add an Admin route:
  `POST /admin/v1/libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-apply`.
- Require the existing Admin route guard.
- Add an Admin request DTO with:
  - `duplicate_source_id`;
  - `expected_action`, limited in this slice to `suggest_relationship`.
- Reuse `SourceDuplicateReconciliationAppService` as the only app-service
  authority for validation and mutation.
- Validate the same redaction-safe plan facts before writing:
  - target source exists and belongs to `library_id`;
  - candidate source exists in the same Media Library;
  - both sources have supported redacted Source Fingerprint evidence;
  - evidence matches and is not stale;
  - the current recommended action is exactly `suggest_relationship`;
  - no existing Suggested, Confirmed, or Rejected relationship is overwritten.
- Create or preserve only a `SourceDuplicateRelationshipStatus::Suggested`
  relationship with canonical source pair ordering.
- Return a redaction-safe Admin response with Admin API version, target source
  id, duplicate source id, relationship id/status, applied action, and whether a
  new relationship was created.
- Keep response and errors free of Source Locators, local paths, raw hashes,
  raw fingerprints, evidence values, etags, backend URLs, credentials, and job
  input JSON.
- Register the route in the Admin generated contract and refresh generated
  Admin TypeScript contracts.
- Update the source hash/reconciliation Trellis spec to document this explicit
  first apply boundary.

## Acceptance Criteria

- [ ] Admin can apply a fresh plan candidate and a Suggested Source Duplicate
      Relationship is persisted.
- [ ] Replaying the same apply does not create duplicate relationship rows and
      reports the existing Suggested relationship safely.
- [ ] Existing Suggested, Confirmed, and Rejected relationships are not
      overwritten.
- [ ] Stale target or candidate evidence recommends refresh and is rejected
      without writing.
- [ ] Cross-library, missing candidate, missing fingerprint, raw fingerprint,
      mismatched fingerprint, and non-admin calls fail without unsafe leaks.
- [ ] The route is covered by Admin route inventory and generated Admin
      contract tests.

## Out Of Scope

- No automatic source hash completion writer.
- No scan-originated reconciliation scheduling.
- No auto-confirmed duplicate relationship.
- No reopening rejected relationships.
- No relationship history/audit/undo model.
- No Media Source merge, Media Item merge, Playback Source Selection change, or
  Library Access change.
- No Public Client API.
- No Admin Web page implementation beyond generated contract refresh.
- No schema migration.

## Technical Approach

- Add minimal core result/request domain records only if shared by API and
  server; otherwise keep orchestration request types in `nako-server` and wire
  DTOs in `nako-api`.
- Add `SourceDuplicateReconciliationAppService::apply_source_duplicate_reconciliation`.
- Reuse existing fingerprint parsing and candidate action logic from the
  read-only plan instead of duplicating policy in HTTP.
- Use `SourceDuplicateRepository::upsert_source_duplicate_relationship` only
  after validating the current action is `suggest_relationship`.
- Add focused app-service tests first, then Admin route tests.

## Validation

- `cargo check -p nako-core -p nako-api -p nako-server --tests`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo nextest run -p nako-server source_duplicate --no-fail-fast`
- `cargo nextest run -p nako-server admin_v1_source_duplicate_reconciliation --no-fail-fast`
- `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-admin-source-duplicate-reconciliation-apply-first-slice`
