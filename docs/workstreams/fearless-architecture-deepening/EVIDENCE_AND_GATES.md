# Fearless Architecture Deepening — Evidence And Gates

Status: Active
Last updated: 2026-05-20

This file records evidence for M63. Do not claim the fearless refactor lane is
complete without fresh command evidence matching the touched Interfaces.

## Gate Policy

Always-on gates:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- focused `cargo nextest run` for touched crates and behavior families
- `git diff --check`

Closeout gates:

- `cargo nextest run --workspace --no-fail-fast`
- PostgreSQL opt-in contract runs for any touched persistence seam when
  `TARU_TEST_POSTGRES_URL` is available.

PostgreSQL opt-in policy:

- SQLite remains always-on.
- PostgreSQL contracts must fail fast when `TARU_TEST_POSTGRES_URL` is absent
  rather than reporting false green evidence.
- New persistence commit seams must have backend-neutral contracts unless the
  workstream explicitly splits a follow-on and gates runtime exposure.

Safety gates:

- No Addon Side Effect refactor may expose raw Addon Tokens, Source Locators,
  storage URIs, local paths, cache URIs, raw source URLs, content hashes,
  database URLs, credentials, or raw database errors in public/admin/addon DTOs.
- No NFO/Library File Write change may bypass Taru-owned VFS write policy,
  backup policy, permission checks, or audit/apply outcome recording.
- No AI/vector/search change may overwrite Canonical Metadata without the
  Generated Artifact and Acceptance Workflow authority described in
  `CONTEXT.md`.
- Reference repositories under `repo-ref/` remain study material only.

## Evidence

### 2026-05-20 — FAD-010 Workstream Opened

Status: complete.

Evidence:

- Created `docs/workstreams/fearless-architecture-deepening/`.
- Recorded architecture review findings after M62 PostgreSQL Production
  Readiness closeout.
- Selected FAD-020 Addon Side Effect Module depth as the first executable task.
- Documented non-goals to keep provider breadth, network traversal, native
  plugin ABI, adaptive bitrate, Managed Artwork PostgreSQL parity, and AI
  runtime out of this lane unless explicitly split back in.

Validation:

```bash
git diff --check
```

Result:

- `git diff --check` passed later during the FAD-020 verification pass with Git
  CRLF normalization warnings only.

### 2026-05-20 — FAD-020 Addon Side Effect Module Depth

Status: complete.

Implementation evidence:

- Kept `crates/taru-server/src/app/addons.rs` as the root
  `AddonAppService` Module for addon registration, token lifecycle, and grant
  administration.
- Split Addon Principal and grant normalization into
  `crates/taru-server/src/app/addons/principal.rs`.
- Split Addon Side Effect intake, idempotency, safe validation error mapping,
  and authority/target validation into
  `crates/taru-server/src/app/addons/intake.rs`.
- Added an Addon Side Effect apply router in
  `crates/taru-server/src/app/addons/side_effect_apply.rs`.
- Split domain-specific apply Adapters:
  - `metadata_write.rs` for Canonical Metadata patch/merge plus existing
    catalog/search refresh behavior;
  - `library_file_write.rs` for NFO Library File Write export through the
    existing VFS/NFO service and backup policy;
  - `artwork_write.rs` for Addon Artwork Candidate proposal;
  - `target.rs` for shared Media Item resolution from side-effect targets.
- No public/admin/addon DTO shape was changed.
- No persistence schema, repository contract, or behavior semantics were
  changed in this slice.

Validation:

```bash
cargo nextest run -p taru-server addon_side_effect --no-fail-fast
cargo fmt --all
cargo check -p taru-server --tests
cargo nextest run -p taru-server addon_side_effect --no-fail-fast
cargo nextest run -p taru-server addon --no-fail-fast
cargo fmt --all -- --check
cargo check -p taru-server --tests
git diff --check
```

Result:

- Baseline focused Addon Side Effect nextest passed before the refactor:
  10 passed, 165 skipped.
- `cargo fmt --all` passed after the refactor.
- `cargo check -p taru-server --tests` passed after the refactor.
- Focused Addon Side Effect nextest passed after the refactor:
  10 passed, 165 skipped.
- Broader addon HTTP nextest passed after the refactor:
  31 passed, 144 skipped.
- `cargo fmt --all -- --check` passed.
- Final `cargo check -p taru-server --tests` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

Broader gates not run:

- Full workspace nextest was not run for FAD-020 because this task is a
  behavior-preserving server Module split, the touched runtime surface is
  covered by focused Addon Side Effect and broader addon HTTP tests, and no
  public API, repository, or persistence contract changed.
- PostgreSQL opt-in contracts were not applicable for FAD-020 because no
  persistence seam or SQL behavior changed. FAD-030 will require DB contract
  evidence if it introduces a transactional Addon metadata commit seam.

### 2026-05-20 — FAD-030 Addon Metadata Commit Atomicity

Status: complete.

Implementation evidence:

- Added `AddonMetadataWritePersistenceCommit` and
  `AddonMetadataWriteCatalogCommit` as Addon-domain workflow-shaped
  persistence seams.
- SQLite and PostgreSQL now commit Addon Canonical Metadata writes in one
  transaction:
  - `MediaItem` mutation;
  - optional Catalog Item Graph replacement;
  - Search Projection upsert;
  - Addon Side Effect `Applied` outcome recording.
- Refactored Addon apply routing so `metadata_write` returns the already
  recorded side-effect outcome instead of issuing a second apply-outcome write.
- Added public catalog planning helpers for search-only and label-projection
  planning so callers can build a commit without prematurely mutating storage.
- Added a backend-neutral contract covering:
  - scalar/search-only metadata writes preserving existing graph labels;
  - label/graph writes replacing the touched Catalog Item Graph and Search
    Projection together;
  - apply outcome source/item/report recording;
  - rollback when Catalog Graph persistence fails after the item mutation was
    attempted.

Validation:

```bash
cargo fmt --all
cargo check -p taru-core -p taru-db -p taru-server --tests
cargo nextest run -p taru-db addon_metadata_write --no-fail-fast
cargo nextest run -p taru-server addon_side_effect --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Result:

- `cargo fmt --all` passed.
- `cargo check -p taru-core -p taru-db -p taru-server --tests` passed.
- Focused SQLite contract passed:
  1 passed, 102 skipped.
- Focused Addon Side Effect nextest passed:
  10 passed, 165 skipped.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

PostgreSQL opt-in:

- Not run in this environment because `TARU_TEST_POSTGRES_URL` was unset.
- The PostgreSQL pair was added:
  `postgres_metadata_catalog_contract_addon_metadata_write_updates_projection_apply_outcome_and_rolls_back`.
  Run it with ignored tests enabled once a test database URL is available.

Broader gates not run:

- Full workspace nextest was not run for FAD-030 because the touched behavior is
  covered by the backend-neutral DB contract and focused Addon Side Effect
  server tests. The full workspace nextest remains a M63 closeout gate.

### 2026-05-20 — FAD-040 Library Ingestion Workflow Depth

Status: complete.

Implementation evidence:

- Deleted the broad caller-facing `LibraryIndexRepository` trait alias from
  `crates/taru-library/src/index.rs`.
- Added `LibraryIngestionWorkflow` in
  `crates/taru-library/src/ingestion.rs` as the workflow-shaped seam for
  Library ingestion.
- `LibraryIndexService` now depends on scanner output plus the workflow seam
  only. It no longer directly coordinates low-level repository calls for
  Source State, Library Item State, Local Inference Evidence, ingestion failure
  resolution, Search Projection planning, or scan-source persistence.
- The workflow Adapter owns:
  - library upsert and scan snapshot lifecycle;
  - scan failure recording;
  - directory snapshot persistence plus scan-failure resolution;
  - source locator lookup and inserted/updated disposition;
  - Local Inference planning;
  - confirmed item preservation and provisional hierarchy reuse/creation;
  - Source State, Library Item State, Local Inference Evidence, Search
    Projection planning, and failure-resolution commit composition;
  - the existing `commit_library_scan_source` persistence seam;
  - missing-source tombstoning after complete non-stale scans.
- Added `index_service_uses_workflow_port_without_repository_traits` as a
  deletion-test style unit proving the index service can run against a fake
  workflow port without any low-level repository trait implementation.
- Preserved the M62 DB transaction seam and its backend-neutral contract tests.

Validation:

```bash
cargo fmt --all
cargo check -p taru-library -p taru-db --tests
cargo nextest run -p taru-library index_service_uses_workflow_port_without_repository_traits --no-fail-fast
cargo nextest run -p taru-db scan_commit --no-fail-fast
cargo nextest run -p taru-library --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Result:

- `cargo fmt --all` passed.
- `cargo check -p taru-library -p taru-db --tests` passed.
- Focused workflow deletion test passed:
  1 passed, 17 skipped.
- Focused SQLite scan commit contracts passed:
  2 passed, 101 skipped.
- Focused `taru-library` nextest passed:
  18 passed, 0 skipped.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

PostgreSQL opt-in:

- Not run in this environment because `TARU_TEST_POSTGRES_URL` was unset.
- Existing ignored PostgreSQL scan commit contract pair remains the opt-in
  parity gate for this seam.

Broader gates not run:

- Full workspace nextest was not run for FAD-040 because the touched behavior is
  covered by the `taru-library` focused run plus backend-neutral scan commit
  contracts. The full workspace nextest remains a M63 closeout gate.

## Evidence To Add During Execution

Each task should add:

- command line used;
- result summary;
- touched Interface or Module;
- whether PostgreSQL opt-in evidence was run, skipped, or not applicable;
- remaining risks and split decisions.
