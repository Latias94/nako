# Source Duplicate Relationship Idempotent Pair Upsert

## Goal

Make `SourceDuplicateRelationship` upsert idempotent by canonical source pair
across SQLite and PostgreSQL, so repeated writers cannot create duplicate rows
for the same pair before source-hash-backed reconciliation is introduced.

## Requirements

- Preserve the existing canonical pair rule:
  - `source_id` and `duplicate_source_id` are ordered through
    `SourceDuplicateRelationship::canonicalized`.
  - a relationship requires two distinct media sources.
- Treat `(source_id, duplicate_source_id)` as the repository identity for
  upsert idempotency, not only the generated relationship id.
- A second upsert for the same canonical pair with a different relationship id
  must update the existing row rather than inserting a duplicate.
- The updated row should reflect the latest relationship payload:
  `evidence_kind`, `evidence_value`, `status`, and `confidence_milli`.
- Keep `SourceDuplicateRelationshipId` stable for an existing pair when the
  upsert is pair-conflicted; callers that already hold the original id should
  continue to resolve it.
- Align PostgreSQL with SQLite through schema and upsert behavior.
- Add a backend-agnostic repository contract test that runs for SQLite and the
  optional `NAKO_TEST_POSTGRES_URL` PostgreSQL contract.
- Keep this change below the API/app boundary. Do not add Admin/Public routes,
  source hash reconciliation, or duplicate relationship apply behavior.

## Acceptance Criteria

- [ ] SQLite contract proves repeated same-pair upsert stores one row.
- [ ] PostgreSQL contract has the same test and is runnable under
      `NAKO_TEST_POSTGRES_URL`.
- [ ] PostgreSQL baseline and incremental migrations enforce canonical pair
      uniqueness.
- [ ] SQLite upsert still passes and remains pair-idempotent.
- [ ] The repository contract proves reversed pair input canonicalizes to the
      same stored row.
- [ ] Existing relationship `id` remains stable after a pair-conflicted upsert.
- [ ] Focused DB checks pass.

## Definition Of Done

- `cargo check -p nako-db --tests` passes.
- `cargo nextest run -p nako-db source_duplicate --no-fail-fast` passes.
- Migration registration tests cover the new migration version.
- `cargo fmt --all -- --check` and `git diff --check` pass.
- Trellis evidence records the validation commands.
- Changes are committed with a Conventional Commit message.

## Out Of Scope

- No automatic source duplicate reconciliation writer.
- No Admin or Public Client API changes.
- No hash evidence detail route.
- No cross-library duplicate relationship semantics.
- No relationship history/audit/undo model.
- No merge of Media Sources or Media Items.

## Technical Approach

- Extend `crates/nako-db/src/contract_tests.rs` with a source duplicate
  repository contract for pair-idempotent upsert.
- Change SQLite upsert to conflict on `(source_id, duplicate_source_id)` while
  keeping the existing row id stable during pair conflict updates.
- Add PostgreSQL pair uniqueness to baseline and an incremental migration for
  migrated stores.
- Change PostgreSQL upsert to conflict on `(source_id, duplicate_source_id)`
  and keep row id stable on pair-conflicted updates.
- Register migration version 5 for SQLite and PostgreSQL if schema files are
  needed.

## Decision (ADR-lite)

**Context**: Source fingerprint hash evidence can later produce duplicate
relationship suggestions. PostgreSQL currently allows repeated rows for the
same canonical pair, while SQLite already treats the pair as unique.

**Decision**: Make pair-level idempotency an explicit repository contract before
adding any automatic or repeated source-hash reconciliation writer.

**Consequences**: Future reconciliation can safely retry/apply suggestions
without creating duplicate pair rows. Relationship ids remain stable for
existing pairs, and pair payloads can be updated by explicit later writes.

## Technical Notes

- Parent wave:
  `.trellis/tasks/06-06-06-06-fearless-refactor-development-wave/`
- Source hash policy spec:
  `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`
- Predecessor audit:
  `.trellis/tasks/archive/2026-06/06-05-source-hash-triggering-reconciliation-policy/research/source-hash-triggering-reconciliation-policy.md`
