# Fearless Architecture Deepening — Evidence And Gates

Status: Completed
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

### 2026-05-20 — FAD-050 Playback And Transcode Identity

Status: complete.

Implementation evidence:

- Added `PlaybackProfileIdentity` in `taru-streaming` while preserving
  `PlaybackProfile::identity_key()` as a compatibility helper.
- Added source-bound transcode identity in `taru-transcode`:
  - `TranscodeSourceIdentity` hashes source revision inputs;
  - `TranscodeRequestIdentity` binds source revision to
    `TranscodeProfileIdentity`;
  - request identity has its own persisted request key and storage slug.
- Updated remux and HLS app services so persisted session request keys,
  duplicate detection, finished-output reuse, and staging paths use
  `TranscodeRequestIdentity` instead of profile-only identity.
- Updated app and HTTP tests to compute expected request keys from the actual
  `MediaSource` revision.
- Added tests proving:
  - transcode request identity changes when source revision changes;
  - HLS selected hardware policy still separates cache/session identity;
  - existing playback route/session behavior remains compatible.

Validation:

```bash
cargo fmt --all
cargo check -p taru-streaming -p taru-transcode -p taru-server --tests
cargo nextest run -p taru-transcode transcode_request_identity --no-fail-fast
cargo nextest run -p taru-streaming playback_profile_identity --no-fail-fast
cargo nextest run -p taru-server hls_source_request_identity --no-fail-fast
cargo nextest run -p taru-streaming -p taru-transcode --no-fail-fast
cargo nextest run -p taru-server playback --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Result:

- `cargo check -p taru-streaming -p taru-transcode -p taru-server --tests`
  passed.
- Focused transcode request identity test passed:
  1 passed, 24 skipped.
- Focused playback profile identity test passed:
  1 passed, 8 skipped.
- Focused server HLS request identity tests passed:
  2 passed, 174 skipped.
- Focused `taru-streaming` + `taru-transcode` nextest passed:
  34 passed, 0 skipped.
- Focused `taru-server playback` nextest passed:
  48 passed, 128 skipped.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

Broader gates not run:

- Full workspace nextest was not run for FAD-050 because the touched behavior is
  covered by focused streaming/transcode/server playback identity tests. The
  full workspace nextest remains a M63 closeout gate.

### 2026-05-20 — FAD-060 Hardware Diagnostics

Status: complete.

Implementation evidence:

- Replaced the single `HardwareCapabilityEvidence` field with separate
  diagnostics records in `taru-transcode`:
  - `HardwareEncoderDiscovery` for static FFmpeg encoder discovery;
  - `HardwareDeviceInitialization` for device initialization evidence;
  - `HardwareSmokeProbe` for optional encode smoke-probe results.
- Added `HardwareDeviceInitializationDetector`,
  `OperatorHardwareDeviceInitialization`, and
  `StaticHardwareDeviceInitialization` so tests can prove device-init outcomes
  without opening privileged host devices.
- Kept normal FFmpeg capability detection based on `ffmpeg -encoders`; default
  device initialization and smoke probes remain operator-guidance `not_run`
  records rather than implicit privileged probes.
- Made explicit device-initialization failures and smoke-probe failures mark
  the accelerator unavailable, while missing encoders and FFmpeg probe errors
  remain distinct reasons.
- Updated Admin playback runtime DTOs to expose safe summaries for encoder
  discovery, device initialization, and smoke probes. Raw FFmpeg errors, device
  paths, local paths, and detail text remain hidden behind `has_detail` booleans
  plus safe `operator_check` guidance.
- Updated the Admin TypeScript contract and admin-web mock playback runtime
  data. The mock system config also gained the already-required database
  diagnostics block so TypeScript checking remains aligned with the generated
  contract.
- Updated `docs/api/HTTP_API.md` for the separate hardware diagnostics layers
  and redaction guarantees.

Validation:

```bash
cargo fmt --all
cargo check -p taru-transcode -p taru-api -p taru-server --tests
cargo nextest run -p taru-transcode hardware --no-fail-fast
cargo nextest run -p taru-transcode --no-fail-fast
cargo nextest run -p taru-api --lib admin_playback_runtime_diagnostics_serializes_safe_summary_fields --no-fail-fast
cargo nextest run -p taru-api --lib admin_contract --no-fail-fast
cargo nextest run -p taru-server admin_v1_playback_runtime_reports_safe_diagnostics --no-fail-fast
npm run check
cargo fmt --all -- --check
git diff --check
```

Result:

- `cargo check -p taru-transcode -p taru-api -p taru-server --tests` passed.
- Focused hardware nextest passed: 6 passed, 20 skipped.
- Full `taru-transcode` nextest passed: 26 passed, 0 skipped.
- Admin playback runtime DTO serialization test passed: 1 passed, 40 skipped.
- Admin TypeScript contract tests passed: 4 passed, 37 skipped.
- Admin playback runtime HTTP diagnostics test passed: 1 passed, 175 skipped.
- `npm run check` in `apps/admin-web` passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

Broader gates not run:

- Full workspace nextest was not run for FAD-060 because the touched behavior is
  covered by full `taru-transcode`, focused Admin API DTO/contract tests, the
  focused server Admin playback runtime route test, and admin-web TypeScript
  checking. The full workspace nextest remains a M63 closeout gate.
- PostgreSQL opt-in contracts were not applicable because FAD-060 changed no
  persistence schema or database commit seam.


### 2026-05-20 — FAD-070 Search Semantics

Status: complete.

Implementation evidence:

- Added a shared search semantics evaluator in `taru-search`:
  - current Search Projection version helpers;
  - `SearchEvaluationDocument` fixtures;
  - exact Browse Facet filtering;
  - title, alias, body, and facet scoring;
  - compact normalized matching so CJK queries are not broken by whitespace.
- SQLite and PostgreSQL `SearchIndex` adapters now load persisted search rows
  and delegate filtering/scoring/pagination to the shared evaluator instead of
  carrying duplicated query semantics in each backend.
- Catalog hydration now loads accepted Provider Mappings, projects their
  Provider Subject title/key into the Search Projection, and emits provider and
  external-id Browse Facets for accepted Provider Subjects.
- `NfoImportRepository` now includes `ProviderMappingRepository` so NFO import
  planning continues to use the richer Catalog Projection seam.
- No AI, vector, FTS, pinyin, romaji, or external search service behavior was
  added in this slice.

Validation:

```bash
cargo fmt --all
$env:TMP='F:\Temp'; $env:TEMP='F:\Temp'; cargo check -p taru-search -p taru-catalog -p taru-db --tests
$env:TMP='F:\Temp'; $env:TEMP='F:\Temp'; cargo check -p taru-nfo -p taru-metadata -p taru-server --tests
$env:TMP='F:\Temp'; $env:TEMP='F:\Temp'; cargo nextest run -p taru-search --no-fail-fast
$env:TMP='F:\Temp'; $env:TEMP='F:\Temp'; cargo nextest run -p taru-catalog semantic_search --no-fail-fast
$env:TMP='F:\Temp'; $env:TEMP='F:\Temp'; cargo nextest run -p taru-db search --no-fail-fast
$env:TMP='F:\Temp'; $env:TEMP='F:\Temp'; cargo nextest run -p taru-db facet --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Result:

- `cargo check -p taru-search -p taru-catalog -p taru-db --tests` passed.
- Downstream bound check passed for `taru-nfo`, `taru-metadata`, and
  `taru-server`.
- Full `taru-search` nextest passed: 6 passed, 0 skipped.
- Focused catalog semantic-search nextest passed: 1 passed, 3 skipped.
- Focused DB search nextest passed: 7 passed, 97 skipped.
- Focused DB facet nextest passed: 1 passed, 103 skipped.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

Environment note:

- `C:\Users\Frankorz\AppData\Local\Temp` reported no free space during
  this slice, causing linker `LNK1108` failures before command execution was
  retried with `TMP`/`TEMP` pointed at `F:\Temp`. The final validation commands
  above used that temporary-directory override where linking test binaries was
  required.

Broader gates not run:

- Full workspace nextest was not run for FAD-070 because the touched behavior is
  covered by `taru-search` evaluator tests, the catalog provider-title semantic
  search fixture, and focused SQLite DB search/facet tests. Full workspace
  nextest remains a M63 closeout gate.
- PostgreSQL opt-in runtime contracts were not run because this task did not add
  a new persistence commit seam and `TARU_TEST_POSTGRES_URL` was not available.

### 2026-05-20 — FAD-080 Test Locality

Status: complete.

Implementation evidence:

- Extracted focused SQLite SearchIndex semantics coverage from the giant
  `crates/taru-db/src/tests.rs` file into
  `crates/taru-db/src/search_tests.rs`.
- Added domain-focused test helpers for:
  - migrated in-memory SQLite stores;
  - Movie Canonical Metadata fixtures;
  - indexed search documents with explicit Browse Facets and aliases.
- Preserved the existing behavior checks for:
  - exact Browse Facet matching;
  - shared CJK-friendly alias semantics;
  - alias search without flattening structured alias fields.
- Left the mixed scan/artwork/search round-trip test in
  `crates/taru-db/src/tests.rs` because it verifies a broader persistence family
  and moving it would be mechanical churn rather than better locality.
- Audited the touched server HTTP/app test families and did not split them in
  this slice because the Addon and workflow tests are coupled to large
  end-to-end router/app fixtures; splitting those without a domain fixture
  redesign would not improve reviewability enough for FAD-080.

Validation:

```bash
cargo fmt --all
$env:TMP='F:\Temp'; $env:TEMP='F:\Temp'; cargo nextest run -p taru-db search --no-fail-fast
$env:TMP='F:\Temp'; $env:TEMP='F:\Temp'; cargo nextest run -p taru-db facet --no-fail-fast
$env:TMP='F:\Temp'; $env:TEMP='F:\Temp'; cargo check --workspace --tests
cargo fmt --all -- --check
git diff --check
```

Result:

- Focused DB search nextest passed: 8 passed, 96 skipped.
- Focused DB facet nextest passed: 1 passed, 103 skipped.
- `cargo check --workspace --tests` passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

Environment note:

- Cargo commands that link or check test artifacts used `TMP`/`TEMP` pointed at
  `F:\Temp` because `C:\Users\Frankorz\AppData\Local\Temp` had no free space.

Broader gates not run:

- Full workspace nextest was not run for FAD-080 because this task only moved
  focused tests and added local fixtures without changing runtime behavior. Full
  workspace nextest remains the FAD-090 closeout gate.
- PostgreSQL opt-in contracts were not applicable because FAD-080 changed no
  persistence seam or SQL behavior.

### 2026-05-20 — FAD-090 Closeout

Status: complete.

Closeout evidence:

- FAD-020 through FAD-090 are complete.
- M4 Search Semantics and Test Locality is complete.
- M5 Closeout Or Split is complete.
- No new follow-on workstream was required during closeout because the remaining
  independent tails already exist as named lanes:
  - `docs/workstreams/managed-artwork-postgresql-parity/`
  - `docs/workstreams/admin-api-typescript-contract/`
- Provider breadth, AI/vector search, network traversal, adaptive playback, and
  client UX remain outside M63 and should be opened as separate product lanes
  when prioritized.

Validation:

```bash
$env:TMP='F:\Temp'; $env:TEMP='F:\Temp'; cargo fmt --all -- --check
$env:TMP='F:\Temp'; $env:TEMP='F:\Temp'; cargo check --workspace --tests
$env:TMP='F:\Temp'; $env:TEMP='F:\Temp'; cargo nextest run --workspace --no-fail-fast
git diff --check
```

Result:

- `cargo fmt --all -- --check` passed.
- `cargo check --workspace --tests` passed.
- Full workspace nextest passed: 498 tests run, 498 passed, 19 skipped.
- `git diff --check` passed.

PostgreSQL opt-in:

- Skipped because `TARU_TEST_POSTGRES_URL` was unset in this environment.
- SQLite always-on coverage and backend-neutral contract pairs remain present.
- PostgreSQL opt-in contract pairs should be run in an environment that provides
  a disposable PostgreSQL test URL.

Environment note:

- Cargo commands used `TMP`/`TEMP=F:\Temp` because the default user temp
  directory had no free space during this workstream.

## Evidence To Add During Execution

Each task should add:

- command line used;
- result summary;
- touched Interface or Module;
- whether PostgreSQL opt-in evidence was run, skipped, or not applicable;
- remaining risks and split decisions.
