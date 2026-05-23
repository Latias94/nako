# Fearless Future Architecture Refactor — TODO

Status: Complete
Last updated: 2026-05-23

Task IDs use the `FFR` prefix.

## M0 — Scope And Boundary Freeze

- [x] FFR-010 [owner=planner] [deps=none] [scope=docs/workstreams/fearless-future-architecture-refactor,repo-ref/jellyfin]
  Goal: Freeze the next fearless refactor lane, record the hotspot map,
  reference policy, and split rules.
  Validation: `docs/workstreams/fearless-future-architecture-refactor/DESIGN.md`,
  `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, and `WORKSTREAM.json`
  exist and agree; `git diff --check`.
  Evidence:
  `docs/workstreams/fearless-future-architecture-refactor/DESIGN.md`;
  `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-010.md`.
  Handoff: Continue with FFR-020.

## M1 — Runtime And Persistence Control Planes

- [x] FFR-020 [owner=codex] [deps=FFR-010] [scope=crates/nako-server/src/app/playback]
  Goal: Split the playback runtime control-plane helpers out of
  `playback/mod.rs` into focused modules without changing playback behavior.
  Validation: `cargo check -p nako-server --tests`; `cargo nextest run -p
  nako-server playback --no-fail-fast`; `git diff --check`.
  Review: No new pass-through wrappers or behavior changes.
  Evidence:
  `crates/nako-server/src/app/playback/staging_policy.rs`;
  `crates/nako-server/src/app/playback/selection.rs`;
  `crates/nako-server/src/app/playback/failure.rs`;
  `crates/nako-server/src/app/playback/events.rs`;
  `crates/nako-server/src/app/playback/paths.rs`;
  `crates/nako-server/src/app/playback/playlist.rs`;
  `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-020.md`.
  Handoff: Continue with FFR-021.

- [x] FFR-021 [owner=codex] [deps=FFR-020] [scope=crates/nako-server/src/app/managed_import.rs,crates/nako-server/src/app/managed_import]
  Goal: Choose the next broad server app module and split one focused runtime
  control-plane slice, prioritizing `addons`, `metadata`, `nfo`, or
  `managed_import` by coupling and test coverage.
  Validation: `cargo check -p nako-server --tests`; focused nextest for the
  touched module; `git diff --check`.
  Review: No new pass-through wrappers or giant helper modules.
  Evidence:
  `crates/nako-server/src/app/managed_import/diagnostics.rs`;
  `crates/nako-server/src/app/managed_import/outcomes.rs`;
  `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-021.md`.
  Handoff: Continue with FFR-030.

- [ ] FFR-030 [owner=codex] [deps=FFR-021] [scope=crates/nako-db,crates/nako-core,crates/nako-server]
  Goal: Split `postgres.rs` and tame the facade into backend and domain
  modules.
  Validation: `cargo check -p nako-db --tests`; focused nextest for contract
  families; PostgreSQL harness when available; `git diff --check`.
  Review: SQLite always on, PostgreSQL opt-in, no fake PostgreSQL layer.
  Evidence: FFR-030A through FFR-030I completed; `postgres.rs` is now mostly
  connection, migration, schema validation, shared helpers, and module
  dispatch. The remaining core backend family is split into
  `postgres/core_catalog.rs`.
  Handoff: Continue with FFR-040.

  - [x] FFR-030A [owner=codex] [deps=FFR-021] [scope=crates/nako-db/src/postgres.rs,crates/nako-db/src/postgres/jobs.rs]
    Goal: Extract PostgreSQL job and job lease persistence from
    `postgres.rs` into a focused backend module while keeping shared managed
    artwork job transaction helpers available through an internal boundary.
    Validation: `cargo check -p nako-db --tests`; `cargo nextest run -p
    nako-db job_lease --no-fail-fast`; `cargo nextest run -p nako-db job
    --no-fail-fast`; `git diff --check`.
    Review: No fake PostgreSQL layer; SQLite remains always-on; PostgreSQL
    contracts remain opt-in.
    Evidence:
    `crates/nako-db/src/postgres/jobs.rs`;
    `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-030a.md`.
    Handoff: Continue with FFR-030B.

  - [x] FFR-030B [owner=codex] [deps=FFR-030A] [scope=crates/nako-db/src/postgres.rs,crates/nako-db/src/postgres/events.rs]
    Goal: Extract PostgreSQL event outbox and webhook persistence into a
    focused backend module with local SQL select fragments, row mapping, and
    delivery-attempt helpers.
    Validation: `cargo check -p nako-db --tests`; focused nextest for the
    touched contract family; PostgreSQL harness when `NAKO_TEST_POSTGRES_URL`
    is available; `git diff --check`.
    Review: No pass-through helper modules; each extracted module owns a
    meaningful repository family and any local SQL/row mapping it needs.
    Evidence:
    `crates/nako-db/src/postgres/events.rs`;
    `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-030b.md`.
    Handoff: Continue with FFR-030C.

  - [x] FFR-030C [owner=codex] [deps=FFR-030B] [scope=crates/nako-db/src/postgres.rs,crates/nako-db/src/postgres/vfs_staging.rs]
    Goal: Extract PostgreSQL VFS cache and staging manifest persistence into a
    focused backend module with local SQL select fragments, row mapping,
    budget accounting, and lease state transitions.
    Validation: `cargo check -p nako-db --tests`; focused nextest for the
    touched contract family; PostgreSQL harness when `NAKO_TEST_POSTGRES_URL`
    is available; `git diff --check`.
    Review: No pass-through helper modules; each extracted module owns a
    meaningful repository family and any local SQL/row mapping it needs.
    Evidence:
    `crates/nako-db/src/postgres/vfs_staging.rs`;
    `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-030c.md`.
    Handoff: Continue with FFR-030D.

  - [x] FFR-030D [owner=codex] [deps=FFR-030C] [scope=crates/nako-db/src/postgres.rs,crates/nako-db/src/postgres/addons_automation.rs]
    Goal: Extract PostgreSQL addon and automation persistence into a focused
    backend module because it mixed addon registration, routing, side-effect
    validation, generated artifact persistence, and automation proposal joins
    in the broad `postgres.rs` backend.
    Validation: `cargo check -p nako-db --tests`; focused nextest for addon,
    automation, and event/addon/automation contract coverage; PostgreSQL
    harness when `NAKO_TEST_POSTGRES_URL` is available; `git diff --check`.
    Review: Preserve side-effect idempotency and proposal diagnostics; do not
    create pass-through helper modules.
    Evidence:
    `crates/nako-db/src/postgres/addons_automation.rs`;
    `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-030d.md`.
    Handoff: Continue with FFR-030E.

  - [x] FFR-030E [owner=codex] [deps=FFR-030D] [scope=crates/nako-db/src/postgres.rs,crates/nako-db/src/postgres/managed_artwork.rs]
    Goal: Extract the next PostgreSQL backend domain module, prioritizing
    managed artwork and artwork candidate persistence because it still owns a
    large transaction-heavy flow around artwork tasks, candidate intake,
    ingest lifecycle, managed artifacts, selection publication, gallery
    hydration, and cleanup.
    Validation: `cargo check -p nako-db --tests`; focused nextest for managed
    artwork and addon artwork candidate contract coverage; PostgreSQL harness
    when `NAKO_TEST_POSTGRES_URL` is available; `git diff --check`.
    Review: Keep job transaction helpers and side-effect outcome helper usage
    explicit across module boundaries; do not introduce fake PostgreSQL layers.
    Evidence: modular backend tree and focused contract tests.
    Handoff: Continue FFR-030 or move to FFR-040 only after the persistence
    width is no longer the highest-risk refactor.

  - [x] FFR-030F [owner=codex] [deps=FFR-030E] [scope=crates/nako-db/src/postgres.rs,crates/nako-db/src/postgres/import_state.rs]
    Goal: Extract the next PostgreSQL backend domain module, prioritizing
    managed import, acquisition intake, and NFO sidecar apply persistence
    because they form a cohesive import-state family with adjacent SQL select
    fragments, repository impls, state transitions, row mapping, and direct
    contract coverage.
    Validation: `cargo check -p nako-db --tests`; focused nextest for
    managed import, acquisition intake, and NFO sidecar apply contracts;
    PostgreSQL harness for all contracts when the slice touches shared import
    transaction helpers; `git diff --check`.
    Review: Keep promotion apply, acquisition link, and NFO audit state
    transitions inside one meaningful backend module; do not split by table if
    that creates pass-through wrappers.
    Evidence: modular backend tree and focused contract tests.
    Handoff: Continue FFR-030 while `postgres.rs` remains a broad backend
    adapter, then move to FFR-040 only after the persistence width is no
    longer the highest-risk refactor.

  - [x] FFR-030G [owner=codex] [deps=FFR-030F] [scope=crates/nako-db/src/postgres.rs,crates/nako-db/src/postgres/metadata_catalog.rs]
    Goal: Extract the next PostgreSQL backend domain module, prioritizing the
    metadata/catalog graph persistence family because `MetadataRepository`,
    `CatalogRepository`, provider mapping, raw response, provider attempt,
    field lock, catalog graph replacement, external IDs, people, credits,
    genres, tags, collections, studios, and image assets still share one broad
    backend file with large transaction helpers.
    Validation: `cargo check -p nako-db --tests`; focused nextest for
    metadata catalog contracts; PostgreSQL all-contract harness because this
    slice touches shared catalog graph transaction helpers; `git diff --check`.
    Review: Split by ownership boundary, not table count. Keep graph
    replacement atomic and avoid moving row mappers away from the SQL family
    that owns them.
    Evidence: modular backend tree and focused metadata/catalog contract
    tests.
    Handoff: Continue FFR-030 while `postgres.rs` remains a broad backend
    adapter, then move to FFR-040 only after persistence width is no longer
    the highest-risk refactor.

  - [x] FFR-030H [owner=codex] [deps=FFR-030G] [scope=crates/nako-db/src/postgres.rs,crates/nako-db/src/postgres/playback_runtime.rs]
    Goal: Decide and execute the next remaining PostgreSQL tail split,
    prioritizing source duplicate plus catalog governance, playback plus
    transcode runtime state, or library/media/scan primitives based on
    cohesion and focused contract coverage.
    Validation: `cargo check -p nako-db --tests`; focused nextest for the
    touched contract family; PostgreSQL all-contract harness when the split
    touches shared row mapping or codec helpers; `git diff --check`.
    Review: Do not keep splitting if the remaining `postgres.rs` width is
    mostly stable infrastructure. Either extract a meaningful family or
    document why FFR-030 can hand off to FFR-040.
    Evidence:
    `crates/nako-db/src/postgres/playback_runtime.rs`;
    `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-030h.md`.
    Judgment: Continue FFR-030 with one more meaningful split because
    `postgres.rs` still owns the cohesive core library/media/scan/search,
    local inference, ingestion failure, source duplicate, and catalog
    governance backend family.
    Handoff: Continue with FFR-030I.

  - [x] FFR-030I [owner=codex] [deps=FFR-030H] [scope=crates/nako-db/src/postgres.rs,crates/nako-db/src/postgres/core_catalog.rs]
    Goal: Extract PostgreSQL core library/media/scan/search persistence from
    `postgres.rs` into a focused backend module. The module should own
    `LibraryRepository`, `LibraryItemRepository`, `MediaRepository`,
    `MediaProbeRepository`, `LocalInferenceRepository`,
    `IngestionFailureRepository`, `ScanRepository`,
    `SearchIndexRepository`, `SourceDuplicateRepository`, and
    `CatalogGovernanceRepository` when the implementation stays cohesive.
    Validation: `cargo check -p nako-db --tests`; focused nextest for
    library/media, scan commit, catalog governance, and ingestion failure
    contracts; PostgreSQL all-contract harness because the split touches
    shared media/source/search transaction helpers and row mapping;
    `git diff --check`.
    Review: Keep low-level connection, migration, schema validation, generic
    row access, numeric conversion, and truly shared codecs in `postgres.rs`.
    Do not split by table if that creates pass-through wrappers.
    Evidence:
    `crates/nako-db/src/postgres/core_catalog.rs`;
    `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-030i.md`.
    Handoff: Move to FFR-040 now that `postgres.rs` is primarily
    infrastructure and module dispatch.

## M2 — API Boundary Split

- [ ] FFR-040 [owner=codex] [deps=FFR-020,FFR-030] [scope=crates/nako-api,docs/api]
  Goal: Split Admin and Public DTO surfaces and keep redaction local to the
  API boundary.
  Validation: `cargo check -p nako-api --tests`; focused nextest for admin
  contract and public client tests; `git diff --check`.
  Review: DTOs must stay explicit and must not mirror DB internals for
  convenience.
  Evidence: FFR-040A completed the first admin surface splits:
  `crates/nako-api/src/admin/playback.rs` and
  `crates/nako-api/src/admin/network.rs`. FFR-040B split storage and staging
  diagnostics into `crates/nako-api/src/admin/storage.rs`. FFR-040C split
  automation/generated artifact DTOs into
  `crates/nako-api/src/admin/automation.rs`. FFR-040D split intake and
  watch-folder discovery DTOs into `crates/nako-api/src/admin/intake.rs`.
  FFR-040E split jobs, outbox, and ingestion failures into
  `crates/nako-api/src/admin/operations.rs`. FFR-040F split catalog
  governance and local inference evidence summaries into
  `crates/nako-api/src/admin/catalog_governance.rs`.
  Handoff: FFR-040 is ready for review; remaining `admin.rs` width is config
  diagnostics and overview summaries.

  - [x] FFR-040A [owner=codex] [deps=FFR-030I] [scope=crates/nako-api/src/admin.rs,crates/nako-api/src/admin]
    Goal: Extract meaningful admin API surfaces from `admin.rs`, starting
    with playback and network because they own redaction, readiness summaries,
    and support evidence rather than passive DTO aliases.
    Validation: `cargo check -p nako-api --tests`; focused nextest for
    playback, network, admin contract, and public client tests; full
    `cargo nextest run -p nako-api --no-fail-fast`; `git diff --check`.
    Review: Keep DTOs explicit and keep redaction local to the moved surface;
    do not introduce pass-through DTO modules.
    Evidence:
    `crates/nako-api/src/admin/playback.rs`;
    `crates/nako-api/src/admin/network.rs`;
    `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-040a.md`.
    Handoff: Continue FFR-040 with config diagnostics or storage/staging
    diagnostics.

  - [x] FFR-040B [owner=codex] [deps=FFR-040A] [scope=crates/nako-api/src/admin.rs,crates/nako-api/src/admin/storage.rs]
    Goal: Extract storage-facing admin DTOs and conversion/redaction tests
    from `admin.rs` into a focused API surface.
    Validation: `cargo check -p nako-api --tests`; focused nextest for
    storage, admin contract, and public client tests; full `cargo nextest run
    -p nako-api --no-fail-fast`; `git diff --check`.
    Review: Keep staging source/local paths and storage credentials redacted;
    keep runtime state as summaries rather than raw backend internals.
    Evidence:
    `crates/nako-api/src/admin/storage.rs`;
    `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-040b.md`.
    Handoff: Continue FFR-040 with config diagnostics or automation/intake
    DTOs.

  - [x] FFR-040C [owner=codex] [deps=FFR-040B] [scope=crates/nako-api/src/admin.rs,crates/nako-api/src/admin/automation.rs]
    Goal: Extract generated artifact and automation review DTOs from
    `admin.rs` into a focused API surface that owns payload/provenance
    summaries and metadata-authority boundary redaction.
    Validation: `cargo check -p nako-api --tests`; focused nextest for
    generated artifact, admin contract, and public client tests; full
    `cargo nextest run -p nako-api --no-fail-fast`; `git diff --check`.
    Review: Do not expose raw prompt JSON, artifact JSON, raw source
    locators, source fingerprints, or secret environment values.
    Evidence:
    `crates/nako-api/src/admin/automation.rs`;
    `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-040c.md`.
    Handoff: Continue FFR-040 with acquisition/watch-folder intake or config
    diagnostics, or move to FFR-050 if review accepts the remaining
    `admin.rs` width as lower risk.

  - [x] FFR-040D [owner=codex] [deps=FFR-040C] [scope=crates/nako-api/src/admin.rs,crates/nako-api/src/admin/intake.rs]
    Goal: Extract acquisition intake and watch-folder discovery DTOs from
    `admin.rs` into a focused API surface that owns source reference
    redaction, admission diagnostics, and discovery failure summaries.
    Validation: `cargo check -p nako-api --tests`; focused nextest for
    intake, admin contract, and public client tests; full `cargo nextest run
    -p nako-api --no-fail-fast`; `git diff --check`.
    Review: Do not expose raw source URIs, intended locators, display names,
    diagnostics JSON, root URIs, local paths, or token-bearing references.
    Evidence:
    `crates/nako-api/src/admin/intake.rs`;
    `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-040d.md`.
    Handoff: Continue FFR-040 with operational admin surfaces before moving
    to FFR-050.

  - [x] FFR-040E [owner=codex] [deps=FFR-040D] [scope=crates/nako-api/src/admin.rs,crates/nako-api/src/admin/operations.rs]
    Goal: Extract operational admin DTOs for jobs, cancellation requests,
    outbox events, ingestion failures, and ignore requests from `admin.rs`
    into a focused API surface that owns payload/error redaction.
    Validation: `cargo check -p nako-api --tests`; focused nextest for job,
    outbox, ingestion failure, admin contract, and public client tests; full
    `cargo nextest run -p nako-api --no-fail-fast`; `git diff --check`.
    Review: Do not expose raw job input JSON, job summary JSON, raw errors,
    outbox payload JSON, idempotency keys, or last-error values.
    Evidence:
    `crates/nako-api/src/admin/operations.rs`;
    `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-040e.md`.
    Handoff: Continue FFR-040 with catalog governance because it still owns
    local inference evidence redaction.

  - [x] FFR-040F [owner=codex] [deps=FFR-040E] [scope=crates/nako-api/src/admin.rs,crates/nako-api/src/admin/catalog_governance.rs]
    Goal: Extract catalog governance and local inference evidence summaries
    from `admin.rs` into a focused API surface that owns governance issue
    derivation and local inference redaction.
    Validation: `cargo check -p nako-api --tests`; focused nextest for
    catalog governance, admin contract, and public client tests; full `cargo
    nextest run -p nako-api --no-fail-fast`; `git diff --check`.
    Review: Do not expose raw local inference `evidence_value`, raw local
    paths, or provider/database internals.
    Evidence:
    `crates/nako-api/src/admin/catalog_governance.rs`;
    `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-040f.md`.
    Handoff: FFR-040 is ready for review. Move to FFR-050 if review accepts
    leaving config diagnostics and overview summaries in `admin.rs`.

## M3 — VFS And Inference Boundary Split

- [x] FFR-050 [owner=codex] [deps=FFR-040] [scope=crates/nako-vfs,crates/nako-library,crates/nako-naming,crates/nako-nfo,crates/nako-server/src/app/metadata.rs]
  Goal: Deepen local inference, naming, and file-write authority so parser
  evidence stays explainable and VFS primitives stay low level.
  Validation: `cargo check -p nako-vfs --tests`; `cargo check -p nako-library
  --tests`; focused nextest for naming, local inference, NFO, and file-write
  paths; `git diff --check`.
  Review: Conservative inference stays conservative; file-write policy stays
  separate from raw VFS primitives.
  Evidence: FFR-050A split local path authority into
  `crates/nako-vfs/src/local/path_authority.rs`. FFR-050B split local write
  transactions into `crates/nako-vfs/src/local/write_transaction.rs`.
  FFR-050C split local apply/link planning into
  `crates/nako-vfs/src/local/apply_plan.rs`. FFR-050D split local
  cleanup/restore lifecycle handling into
  `crates/nako-vfs/src/local/lifecycle.rs`. FFR-050E removed the
  `nako-naming` dependency on `nako-core` and moved parsed-name to Nako-domain
  mapping into `nako-library`.
  Review: No blocking findings. Remaining `local_inference.rs` width is an
  internal follow-up candidate, not a blocker for this lane.
  Handoff: Move to FFR-060.

  - [x] FFR-050A [owner=codex] [deps=FFR-040F] [scope=crates/nako-vfs/src/local.rs,crates/nako-vfs/src/local/path_authority.rs]
    Goal: Extract local root, URI scheme, relative path, canonical parent,
    and security-violation logic from `local.rs` into a focused local path
    authority module.
    Validation: `cargo check -p nako-vfs --tests`; `cargo nextest run -p
    nako-vfs local --no-fail-fast`; `cargo nextest run -p nako-vfs
    --no-fail-fast`; `git diff --check`.
    Review: Do not change local read, write, link, cleanup, restore, backup,
    or staging behavior.
    Evidence:
    `crates/nako-vfs/src/local/path_authority.rs`;
    `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-050a.md`.
    Handoff: Continue FFR-050 with a local write transaction split or local
    inference/naming boundary split.

  - [x] FFR-050B [owner=codex] [deps=FFR-050A] [scope=crates/nako-vfs/src/local.rs,crates/nako-vfs/src/local/write_transaction.rs]
    Goal: Extract local atomic replace, backup creation, backup retention
    pruning, restore temp-file handling, and fsync helpers from `local.rs`
    into a focused write transaction module.
    Validation: `cargo check -p nako-vfs --tests`; `cargo nextest run -p
    nako-vfs --no-fail-fast`; `git diff --check`.
    Review: Do not change local write, atomic replace, backup, restore,
    link/copy, cleanup, or staging behavior.
    Evidence:
    `crates/nako-vfs/src/local/write_transaction.rs`;
    `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-050b.md`.
    Handoff: Continue FFR-050 with local apply/link planning or local
    inference/naming boundary split.

  - [x] FFR-050C [owner=codex] [deps=FFR-050B] [scope=crates/nako-vfs/src/local.rs,crates/nako-vfs/src/local/apply_plan.rs]
    Goal: Extract local link planning, copy/link apply, apply status mapping,
    and copy/symlink file actions from `local.rs` into a focused apply-plan
    module.
    Validation: `cargo check -p nako-vfs --tests`; `cargo nextest run -p
    nako-vfs local_backend --no-fail-fast`; `cargo nextest run -p nako-vfs
    --no-fail-fast`; `git diff --check`.
    Review: Keep path authority in `path_authority.rs`, write transactions in
    `write_transaction.rs`, and do not change local copy, hardlink, symlink,
    cleanup, restore, or staging behavior.
    Evidence:
    `crates/nako-vfs/src/local/apply_plan.rs`;
    `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-050c.md`.
    Handoff: Continue FFR-050 with local cleanup/restore report boundaries or
    local inference/naming boundary split.

  - [x] FFR-050D [owner=codex] [deps=FFR-050C] [scope=crates/nako-vfs/src/local.rs,crates/nako-vfs/src/local/lifecycle.rs]
    Goal: Extract local cleanup, restore, lifecycle status mapping, and
    request-validation reports from `local.rs` into a focused lifecycle
    module.
    Validation: `cargo check -p nako-vfs --tests`; `cargo nextest run -p
    nako-vfs local_backend --no-fail-fast`; `cargo nextest run -p nako-vfs
    --no-fail-fast`; `git diff --check`.
    Review: Keep path authority, apply/link planning, and write transactions
    in their existing focused modules. Do not change local cleanup, restore,
    backup, apply, read, write, staging, or WebDAV behavior.
    Evidence:
    `crates/nako-vfs/src/local/lifecycle.rs`;
    `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-050d.md`.
    Handoff: Continue FFR-050 with local inference/naming boundary split.

  - [x] FFR-050E [owner=codex] [deps=FFR-050D] [scope=crates/nako-naming,crates/nako-library/src/local_inference.rs,Cargo.lock]
    Goal: Make `nako-naming` a pure naming parser crate by removing its
    dependency on `nako-core`, and keep mapping into Nako `MediaKind` plus
    `LocalInferenceEvidenceSource` inside the library local-inference
    boundary.
    Validation: `cargo check -p nako-naming --tests`; `cargo check -p
    nako-library --tests`; `cargo nextest run -p nako-naming --no-fail-fast`;
    `cargo nextest run -p nako-library local_inference --no-fail-fast`;
    `cargo nextest run -p nako-library --no-fail-fast`; `cargo tree -p
    nako-naming --depth 1`; `git diff --check`.
    Review: Naming parser output must not expose catalog evidence DTOs or
    repository/domain persistence types; library owns the Nako-domain mapping.
    Evidence:
    `crates/nako-naming/src/lib.rs`;
    `crates/nako-library/src/local_inference.rs`;
    `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-050e.md`.
    Handoff: Review FFR-050 for closure or continue with a smaller
    `local_inference.rs` internal split.

## M4 — Docker Validation And Deletion Sweep

- [x] FFR-060 [owner=planner] [deps=FFR-020,FFR-030,FFR-040,FFR-050] [scope=docs,deploy,scripts,workspace]
  Goal: Codify Docker-backed local validation, then delete replaced code paths
  and stale helpers.
  Validation: `cargo fmt --all -- --check`; `cargo check --workspace --tests`;
  `cargo nextest run --workspace --no-fail-fast`; `pwsh -NoProfile
  -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode container`;
  `pwsh -NoProfile -ExecutionPolicy Bypass -File
  scripts/postgres-contract-harness.ps1 -Suite all-contracts`; `git diff --check`.
  Review: Every removed path must have a named replacement or split follow-on.
  Evidence: Deletion sweep found no remaining replaced helper paths requiring
  immediate removal. Closeout gates passed: `cargo fmt --all -- --check`;
  `cargo check --workspace --tests`; `cargo nextest run --workspace
  --no-fail-fast`; `pwsh -NoProfile -ExecutionPolicy Bypass -File
  scripts/release-gate.ps1 -Mode container`; `pwsh -NoProfile
  -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite
  all-contracts`; `git diff --check`.
  Handoff: Workstream complete. Remaining `local_inference.rs` internal width
  is a follow-up candidate, not a blocker.
