# Watch-folder Source-key Normalization Cleanup

## Goal

Normalize historical watch-folder acquisition intake candidate source keys to the canonical `watch_folder:<storage-uri>` form, then remove the runtime legacy fallback from watch-folder discovery. This keeps intake idempotency in storage where it belongs and makes the watch-folder discovery path predictable for the next scheduler/readiness work.

## What I Already Know

* `docs/architecture/LIBRARY_PIPELINE.md` marks watch-folder intake stability as shipped MVP foundation but still calls out watcher/scheduler follow-ups.
* The previous backend readiness audit explicitly said the legacy watch-folder source-key fallback should be kept only until a migration, repair command, or repository-level characterization proves legacy source keys are normalized or impossible.
* Current discovery checks the canonical key first and then recomputes a legacy key from URI, size, and fingerprint before recording a candidate.
* `acquisition_intake_candidates` has a unique key on `(target_library_id, source_kind, source_kind_key, source_key)`, so normalization must define behavior when both legacy and canonical rows exist.
* SQLite and PostgreSQL migrations are registered manually in `nako-db` and currently end at version 5.

## Requirements

* Add a versioned database migration for SQLite and PostgreSQL that normalizes legacy watch-folder acquisition intake candidate source keys to `watch_folder:` plus the stored `source_uri`.
* Preserve canonical rows when both canonical and legacy rows exist for the same `(target_library_id, source_kind, source_kind_key, source_uri)` identity.
* Prefer the most product-meaningful row when duplicates must collapse: keep accepted/managed-import-linked rows ahead of unlinked candidates, then keep the newest observation.
* Keep the migration idempotent and safe for empty databases.
* Remove runtime legacy source-key lookup from watch-folder discovery after the data repair exists.
* Replace the old compatibility test with tests for canonical-only discovery and migration-backed normalization.
* Keep diagnostics redaction intact: no raw local paths, raw storage URI, raw fingerprint string, or legacy `fingerprint=` fragment should leak from watch-folder discovery responses.

## Acceptance Criteria

* [ ] SQLite migration version 6 is registered and applied after version 5.
* [ ] PostgreSQL migration version 6 is registered and applied after version 5.
* [ ] Migration tests cover legacy-only rows and duplicate legacy/canonical rows.
* [ ] Watch-folder discovery no longer calls a legacy source-key helper or queries by a non-canonical key.
* [ ] Acquisition intake tests still prove repeated discovery does not duplicate watch-folder candidates and transitions Inspecting to Ready.
* [ ] Focused Rust tests pass with nextest or cargo test where nextest is unavailable.
* [ ] Only files belonging to this task are staged and committed; existing CRLF-only dirty files remain untouched.

## Technical Approach

Add SQL migrations rather than another app-level compatibility branch. The migration should rewrite legacy watch-folder rows into canonical keys, and when a canonical row already exists, merge meaningful state into a single canonical record before deleting the extra legacy row. Then simplify `find_existing_watch_folder_candidate` to canonical-only lookup and delete `legacy_watch_folder_source_key`.

## Decision (ADR-lite)

**Context**: The old server fallback hides historical key-shape drift inside every discovery tick. That protects old data but makes the runtime path broader than the desired product model.

**Decision**: Repair persisted data with database migrations and remove runtime fallback logic.

**Consequences**: Upgrade correctness moves into schema lifecycle, which is easier to reason about and test. The migration needs careful duplicate handling across SQLite and PostgreSQL because unique source-key constraints can otherwise reject normalization.

## Out of Scope

* OS filesystem watcher daemon.
* Debounce policy redesign.
* Scheduled reconciliation scan behavior.
* Admin UI changes.
* New public API endpoints.

## Technical Notes

* Relevant docs:
  * `docs/architecture/LIBRARY_PIPELINE.md`
  * `.trellis/tasks/archive/2026-06/06-16-06-16-backend-readiness-control-plane-audit/audit.md`
* Relevant code:
  * `crates/nako-server/src/app/acquisition_intake.rs`
  * `crates/nako-server/src/app/tests/acquisition_intake.rs`
  * `crates/nako-db/src/sqlite/migrations.rs`
  * `crates/nako-db/src/postgres.rs`
  * `crates/nako-db/migrations/baseline.sql`
  * `crates/nako-db/migrations/postgres/baseline.sql`
  * `crates/nako-db/src/contract_tests.rs`
