# Core Architecture Deepening TODO

Status: Completed
Last updated: 2026-05-18

## M0 - Open Lane

- [x] CAD-010 [owner=codex] [deps=none] [scope=docs/workstreams/core-architecture-deepening,docs/workstreams/README.md]
  Goal: Open the architecture-first execution lane, record non-goals,
  deletion policy, task ordering, gates, and handoff state.
  Validation: `git diff --check`.
  Evidence: `docs/workstreams/core-architecture-deepening/`.
  Handoff: Continue with CAD-020 before touching implementation code.

## M1 - Durable Commit Units

- [x] CAD-020 [owner=codex] [deps=CAD-010] [scope=crates/nako-core,crates/nako-db,crates/nako-nfo,crates/nako-server/src/app/nfo.rs,docs]
  Goal: Replace NFO import's ordered caller-side writes with one NFO import
  commit interface that owns Media Item, Metadata Field Lock, hierarchy
  confirmation, and any required follow-on projection state.
  Validation: `cargo check -p nako-core --tests`; `cargo check -p nako-db --tests`; `cargo check -p nako-nfo --tests`; focused `cargo nextest run -p nako-nfo --no-fail-fast`; focused `cargo nextest run -p nako-db nfo --no-fail-fast`; `git diff --check`.
  Review: Prove rollback for failures after item update and after lock writes;
  remove replaced production write path.
  Evidence: `NfoImportPersistenceCommit` / `commit_nfo_import` now own item,
  field lock, hierarchy confirmation state, catalog graph, and search
  projection writes; `nako-nfo` import now plans a commit and no longer calls
  `upsert_media_item`, `upsert_field_lock`, `confirm_hierarchy`, or
  `hydrate_item_catalog` in production import flow.
  Handoff: Continue with CAD-030 after the commit interface is the only
  production NFO import path.

- [x] CAD-030 [owner=codex] [deps=CAD-020] [scope=crates/nako-core,crates/nako-db,crates/nako-library,crates/nako-search,docs]
  Goal: Replace scattered discovered-source writes with one Library Scan Source
  Commit that owns Media Item, Media Source, Source State, Library Item State,
  Local Inference Evidence, Search Projection, and scan failure resolution.
  Validation: `cargo check -p nako-library --tests`; `cargo check -p nako-db --tests`; focused `cargo nextest run -p nako-library --no-fail-fast`; focused `cargo nextest run -p nako-db scan --no-fail-fast`; `git diff --check`.
  Review: Prove stale Search Projection or unresolved scan failure cannot
  survive a failed source commit; remove replaced caller-side ordering.
  Evidence: `LibraryScanSourcePersistenceCommit` /
  `commit_library_scan_source` now own Media Item, Media Source, Source State,
  Library Item State, Local Inference Evidence, Search Projection, and scan
  failure resolution writes; `nako-library` now plans provisional hierarchy
  parents without early writes and commits discovered source state in one call.
  Handoff: Continue with CAD-040 once scan source commits are atomic.

## M2 - Workflow Ports And Deletion

- [x] CAD-040 [owner=codex] [deps=CAD-030] [scope=crates/nako-server/src/app,crates/nako-core/src/repository,crates/nako-nfo,crates/nako-library,docs]
  Goal: Narrow application-service dependencies around the newly deepened NFO
  and Library scan interfaces so app services depend on focused workflow ports
  instead of broad `SqliteStore` behavior where the seam now earns its keep.
  Validation: `cargo check -p nako-server --tests`; focused `cargo nextest run -p nako-server app --no-fail-fast`; focused crate tests from CAD-020 and CAD-030; `git diff --check`.
  Review: Avoid mechanical trait splitting; every new interface must hide real
  workflow complexity and improve test locality.
  Evidence: `NfoImportRepository` now names the NFO import workflow port over
  the catalog, library item, media, and metadata repositories required to plan
  and commit NFO imports through `commit_nfo_import`. `LibraryIndexRepository`
  now names the Library indexing workflow port over the repositories required
  to plan and commit discovered-source state through
  `commit_library_scan_source`. The seams hide the new durable commit units
  without adding pass-through methods or retaining replaced caller-side write
  ordering.
  Handoff: Continue with CAD-050 playback/transcode profile identity before the
  broader CAD-080 deletion sweep.

- [x] CAD-080 [owner=codex] [deps=CAD-040,CAD-050,CAD-060,CAD-070] [scope=workspace,docs]
  Goal: Remove obsolete helpers, duplicated write paths, compatibility
  shortcuts, shallow adapters, and dead tests left behind by the deepened
  interfaces.
  Validation: `cargo fmt --all -- --check`; `cargo check --workspace --tests`; focused `cargo nextest run` for every touched crate; `git diff --check`.
  Review: No production old/new parallel paths remain unless a documented
  follow-on owns a migration reason and expiry.
  Evidence: Deletion scan found no remaining production use of the replaced
  NFO import or discovered-source caller-side write ordering. DB persistence
  tests and client mock payloads no longer use obsolete `hls:single` or
  `remux:<container>` request-key examples. CAD-060's fakeable smoke-probe
  hook is now exercised by a transcode test instead of remaining an unused
  boundary.
  Handoff: Continue with CAD-090 closeout.

## M3 - Playback And Transcode Identity

- [x] CAD-050 [owner=codex] [deps=CAD-010] [scope=crates/nako-core,crates/nako-streaming,crates/nako-transcode,crates/nako-server/src/app/playback,crates/nako-server/src/http/playback.rs,docs]
  Goal: Replace weak HLS request identity with Playback Profile / Transcode
  Profile identity that can safely encode client capability, output policy,
  track selection, hardware policy, and reuse semantics.
  Validation: `cargo check -p nako-streaming --tests`; `cargo check -p nako-transcode --tests`; `cargo check -p nako-server --tests`; focused `cargo nextest run -p nako-server playback --no-fail-fast`; `git diff --check`.
  Review: Existing single-variant behavior remains compatible, but persisted
  request keys are built from explicit profile facts instead of constants.
  Evidence: `PlaybackProfile` now normalizes client capabilities, storage
  context, output preferences, track selection, bitrate, and HDR preferences.
  `TranscodeProfile` now produces stable profile request keys and hashed
  storage slugs for Remux and HLS single-variant outputs. HLS persisted request
  identity includes selected hardware acceleration, and HLS/Remux output paths
  include the profile storage slug instead of sharing one source-level output
  directory. The previous `hls:single` and `remux:mp4` production constants are
  no longer used.
  Handoff: Continue with CAD-060 hardware diagnostics.

- [x] CAD-060 [owner=codex] [deps=CAD-050] [scope=crates/nako-transcode,crates/nako-server/src/app/playback,crates/nako-api/src/admin.rs,crates/nako-server/src/http/admin.rs,docs]
  Goal: Deepen hardware acceleration diagnostics beyond FFmpeg encoder listing
  by adding safe capability evidence and smoke-probe hooks or documented
  operator checks for VAAPI, NVENC, and Quick Sync.
  Validation: `cargo check -p nako-transcode --tests`; `cargo check -p nako-api --tests`; `cargo check -p nako-server --tests`; focused `cargo nextest run -p nako-transcode --no-fail-fast`; focused admin playback diagnostics tests; `git diff --check`.
  Review: Diagnostics must not leak local paths, secrets, or unsafe process
  details; CI may use fake detectors while hardware smoke remains operator-run.
  Evidence: `HardwareCapabilityEvidence` now separates CPU availability,
  FFmpeg encoder-listed evidence, encoder-missing evidence, probe errors, and
  static detector fakes. `HardwareSmokeProbe` records safe status plus
  operator-run VAAPI/NVENC/Quick Sync smoke-check text, while the admin API
  exposes only safe evidence, status, operator guidance, and `has_detail`
  instead of raw probe detail. `FfmpegHardwareAccelerationDetector` now accepts
  a fakeable smoke-probe detector for tests and continues to use operator-run
  smoke checks in production.
  Handoff: Continue with CAD-070 addon alignment before the CAD-080 deletion
  sweep.

## M4 - Addon Alignment

- [x] CAD-070 [owner=codex] [deps=CAD-010] [scope=docs/workstreams,crates/nako-core,crates/nako-db,crates/nako-server]
  Goal: Audit Addon Sidecar protected-write and file-write follow-ons against
  the new commit interfaces so addon writes reuse first-party durable modules
  rather than inventing parallel commit paths.
  Validation: docs consistency review; focused checks for touched addon crates
  or server modules; `git diff --check`.
  Review: Do not reimplement addon token, grant, artwork, subtitle, NFO, or
  Library File Write behavior in this lane.
  Evidence: Audit found the only implemented Addon write path is bounded
  `metadata_write`; artwork, subtitle, NFO, and Library File Write runtime
  behavior remains split to dedicated addon workstreams. Those workstreams now
  explicitly require NFO-derived metadata apply to reuse
  `commit_nfo_import`, source/state/search changes to reuse
  `commit_library_scan_source` or a new first-party commit unit, and artwork
  multi-row persistence to reuse or introduce a Nako-owned artwork/catalog
  commit boundary rather than embedding write ordering in Addon handlers.
  Handoff: Continue with CAD-080 deletion sweep.

## M5 - Closeout

- [x] CAD-090 [owner=codex] [deps=CAD-080] [scope=workspace,docs]
  Goal: Close the workstream with refreshed evidence, final docs, and workspace
  gates.
  Validation: `cargo fmt --all -- --check`; `cargo check --workspace --tests`; `cargo nextest run --workspace --no-fail-fast`; `git diff --check`.
  Review: Confirm no task claims completion without fresh command evidence.
  Evidence: Workspace closeout gates passed: `cargo check --workspace --tests`;
  `cargo nextest run --workspace --no-fail-fast` with 375 tests passed;
  `cargo fmt --all -- --check`; `git diff --check` with CRLF-normalization
  warnings only.
  Handoff: Workstream closed. Continue addon artwork or Library File Write
  breadth in their dedicated workstreams, not here.
