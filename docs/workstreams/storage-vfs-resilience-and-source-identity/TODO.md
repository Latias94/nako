# Storage/VFS Resilience And Source Identity — TODO

Status: Completed
Last updated: 2026-05-30

## M0 — Scope And Authority

- [x] SVRS-010 [owner=planner] [deps=none] [scope=docs]
  Goal: Open the durable workstream from the architecture review and link it to
  storage/VFS architecture evidence.
  Validation: workstream docs exist and `WORKSTREAM.json` parses.
  Evidence: `DESIGN.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`,
  `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Start implementation at SVRS-020. Do not touch Web or HLS runtime
  files for this setup slice.

## M1 — Source Identity Evidence Policy

- [x] SVRS-020 [owner=codex] [deps=SVRS-010] [scope=crates/nako-core,crates/nako-library,crates/nako-db]
  Goal: Define and prove the first layered **Source Fingerprint** evidence
  policy for scan commits without forcing full-file hashes.
  Validation:
  `cargo nextest run -p nako-library source_identity scan --no-fail-fast` and
  focused SQLite/PostgreSQL repository contract tests if persistence changes.
  Evidence: `SourceFingerprintEvidence` and `SourceFingerprintPolicyInput` in
  `nako-core`; `VfsLibraryScanner` derives redaction-safe fingerprints from
  layered scan metadata; focused `nako-core` and `nako-library` tests passed.
  Persistence note: no schema or repository contract changed, so database
  contract tests were not required for this slice.
  Review: Check evidence confidence, privacy/redaction, and no accidental
  automatic source merge.
  Handoff: SVRS-030 can use the evidence to reconcile moves and renames.

## M2 — Move/Rename Reconciliation

- [x] SVRS-030 [owner=codex] [deps=SVRS-020] [scope=crates/nako-library,crates/nako-db,crates/nako-server]
  Goal: Preserve **Media Source** and item state across strong-evidence moves or
  renames while keeping weak evidence as reviewable state.
  Validation:
  `cargo nextest run -p nako-library rename_reconciliation --no-fail-fast` and
  `cargo nextest run -p nako-db scan source_duplicate --no-fail-fast`.
  Evidence: `LibraryIndexService` now carries current scan locators into source
  observation commits; strong content-hash relocation reuses the existing
  **Media Source** and item state only when the old locator is absent from the
  current scan; weak and duplicate evidence creates suggested
  **Source Duplicate Relationship** records in the scan commit transaction.
  Persistence note: scan source commits now include duplicate relationships and
  SQLite/PostgreSQL adapters upsert them inside the same atomic scan-source
  unit.
  Review: Confirm tombstones, playback state, provider mappings, and duplicate
  relationships stay coherent.
  Handoff: SVRS-040 can add bounded failure classification around the same
  scan/stage paths.

## M3 — Storage Failure Classification And Backoff

- [x] SVRS-040 [owner=codex] [deps=SVRS-020] [scope=crates/nako-vfs,crates/nako-library,crates/nako-server]
  Goal: Classify timeout, unavailable, permission, rate-limit, stale-cache, and
  partial-read failures consistently across VFS-backed scan/probe/stage paths.
  Validation:
  `cargo nextest run -p nako-vfs --no-fail-fast` and
  `cargo nextest run -p nako-server storage --no-fail-fast`.
  Evidence: `StorageFailureClass` in `nako-core`; WebDAV short-range reads are
  classified as partial reads; VFS cache and library scan/probe failures persist
  redaction-safe messages; library storage backends apply bounded process-local
  backoff to read/probe/stage calls for retryable storage classes only.
  Persistence note: no schema, migration, or repository contract changed.
  Review: Check that storage failures are bounded, redaction-safe, and do not
  hold global locks or unrelated library budgets.
  Handoff: SVRS-050 can expose operator diagnostics and cleanup behavior.

## M4 — Diagnostics And Cleanup

- [x] SVRS-050 [owner=codex] [deps=SVRS-030,SVRS-040] [scope=crates/nako-api,crates/nako-db,crates/nako-server,generated-admin-contracts,docs]
  Goal: Add redaction-safe Admin diagnostics for source identity reconciliation,
  stale VFS cache, storage health, and partial staging cleanup pressure.
  Validation:
  `cargo nextest run -p nako-server system storage --no-fail-fast` and
  `cargo nextest run -p nako-api admin_contract --no-fail-fast` if DTOs change.
  Evidence: Admin overview now includes catalog governance pressure counts;
  storage staging diagnostics include cleanup-candidate record and byte counts;
  storage backend health exposes redaction-safe failure class and backoff
  timestamps; catalog governance includes duplicate-only source-identity
  reconciliation pressure in SQLite/PostgreSQL query paths; Admin TypeScript
  generated contracts were synchronized for the changed DTOs.
  Persistence note: no schema or migration changed. PostgreSQL SQL shape was
  updated and compile-checked, but the opt-in PostgreSQL runtime harness was
  not run because no `NAKO_TEST_POSTGRES_URL` was configured in this workspace.
  Review: Confirm no **Source Locator**, local path, raw ETag, token, or
  fingerprint value leaks.
  Handoff: SVRS-060 closes or splits watcher/debounce and backend-specific
  resilience follow-ons.

## M5 — Closeout And Follow-On Split

- [x] SVRS-060 [owner=planner] [deps=SVRS-050] [scope=docs/workstreams/storage-vfs-resilience-and-source-identity,docs/architecture]
  Goal: Close the lane or split follow-ons such as watcher/debounce,
  backend-specific circuit breakers, and expensive hash policies.
  Validation: final focused gates, `cargo fmt --all -- --check`,
  `cargo check --workspace --tests`, `cargo nextest run --workspace --no-fail-fast`
  when risk justifies it, `git diff --check`, and `WORKSTREAM.json` parse.
  Evidence: `CLOSEOUT.md` records the final shipped behavior, fresh gates, and
  explicit follow-ons; architecture maps now mark this first slice as shipped
  and keep future watcher/debounce, circuit-breaker, hash escalation, and
  PostgreSQL runtime harness work as proposed lanes.
  Review: Run review-workstream and verify-rust-workstream before closeout.
  Handoff: DONE. Open a new workstream for each follow-on before implementation.
