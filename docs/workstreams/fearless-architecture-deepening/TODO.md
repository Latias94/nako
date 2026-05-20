# Fearless Architecture Deepening — TODO

Status: Active
Last updated: 2026-05-20

Task IDs use the `FAD` prefix.

## M0 — Scope And Evidence Freeze

- [x] FAD-010 [owner=codex] [deps=none] [scope=docs/workstreams/fearless-architecture-deepening,docs/GOALS.md,docs/ROADMAP.md,docs/workstreams/README.md]
  Goal: Record the architecture review findings, choose the first execution
  slice, define non-goals, and publish validation gates for the fearless
  refactor lane.
  Validation: `git diff --check`.
  Review: The lane must not become an unbounded feature bucket. Split
  independent product workstreams when the next task no longer shares the same
  architecture problem.
  Evidence: `docs/workstreams/fearless-architecture-deepening/DESIGN.md`;
  `docs/workstreams/fearless-architecture-deepening/EVIDENCE_AND_GATES.md`.
  Progress: Workstream opened after M62 closeout and the 2026-05-20
  architecture review. First executable task selected: FAD-020.
  Handoff: Continue with Addon Side Effect Module depth before adding new
  provider, plugin, AI, or playback breadth.

## M1 — Addon Side Effect Depth

- [x] FAD-020 [owner=codex] [deps=FAD-010] [scope=crates/taru-server/src/app/addons.rs,crates/taru-server/src/app/addons/**,crates/taru-server/src/http/addons.rs,crates/taru-server/src/http/tests/addons.rs]
  Goal: Split Addon Side Effect handling into deeper Modules for principal/grant
  resolution, side-effect intake, apply routing, and domain-specific apply
  Adapters without changing behavior.
  Validation: `cargo check -p taru-server --tests`; focused `cargo nextest run
  -p taru-server addon_side_effect --no-fail-fast`; `git diff --check`.
  Review: The split must improve locality. Do not create pass-through Modules
  that merely rename current function calls.
  Evidence: new Module layout and focused Addon Side Effect tests.
  Progress: Split the former `app/addons.rs` side-effect implementation into
  focused Modules under `app/addons/`: `principal`, `intake`,
  `side_effect_apply`, `metadata_write`, `library_file_write`, `artwork_write`,
  and shared target resolution. The root `AddonAppService` now owns addon
  registration/token/grant administration while side-effect behavior is routed
  through domain-specific apply Adapters. Behavior stayed stable through
  focused Addon Side Effect and broader addon HTTP tests.
  Handoff: Continue with FAD-030 Addon metadata commit atomicity.

- [x] FAD-030 [owner=codex] [deps=FAD-020] [scope=crates/taru-core/src/repository,crates/taru-db,crates/taru-server/src/app/addons/**,docs/workstreams/fearless-architecture-deepening]
  Goal: Add a transactional commit seam for Addon Canonical Metadata writes so
  metadata mutation, Catalog Item Graph/Search Projection consistency, apply
  outcome recording, and rollback behavior are proven together.
  Validation: `cargo check -p taru-core -p taru-db -p taru-server --tests`;
  focused SQLite contract tests; PostgreSQL opt-in contract tests when
  `TARU_TEST_POSTGRES_URL` is available; focused Addon Side Effect nextest;
  `git diff --check`.
  Review: The Interface should express the domain action, not expose a sequence
  of repository calls that callers must order correctly.
  Evidence: backend-neutral contract tests and Addon apply tests.
  Progress: Added the `AddonMetadataWritePersistenceCommit` seam and implemented
  it for SQLite/PostgreSQL so item mutation, optional Catalog Item Graph
  replacement, Search Projection upsert, and Addon Side Effect `Applied`
  outcome are committed in one transaction. Server metadata write apply now
  plans catalog/search projections and delegates the domain commit instead of
  ordering repository calls itself. Added a backend-neutral contract for
  search-only writes, graph writes, apply outcome recording, and rollback.
  Validation: `cargo check -p taru-core -p taru-db -p taru-server --tests`;
  `cargo nextest run -p taru-db addon_metadata_write --no-fail-fast`; `cargo
  nextest run -p taru-server addon_side_effect --no-fail-fast`; `cargo fmt
  --all -- --check`; `git diff --check`. PostgreSQL opt-in was not run because
  `TARU_TEST_POSTGRES_URL` was unset.
  Handoff: Continue with Library ingestion only after Addon write consistency is
  proven or split with a blocker.

## M2 — Library Ingestion Workflow Depth

- [x] FAD-040 [owner=codex] [deps=FAD-030] [scope=crates/taru-library,crates/taru-core/src/repository,crates/taru-db,docs/workstreams/fearless-architecture-deepening]
  Goal: Deepen the Library ingestion commit Interface so scanning and Local
  Inference callers do not need a broad repository trait alias to coordinate
  Source State, Library Item State, Local Inference Evidence, ingestion
  failures, and Search Projection side effects.
  Validation: `cargo check -p taru-library -p taru-db --tests`; focused
  `cargo nextest run -p taru-db scan_commit --no-fail-fast`; focused
  `cargo nextest run -p taru-library --no-fail-fast`; PostgreSQL opt-in scan
  contract when available; `git diff --check`.
  Review: Preserve the M62 scan commit contract. Prefer a workflow-shaped seam
  over mechanical repository trait splitting.
  Evidence: narrowed Library ingestion Interface and contract/test updates.
  Progress: Replaced the broad `LibraryIndexRepository` caller bound with a
  workflow-shaped `LibraryIngestionWorkflow` seam. `LibraryIndexService` now
  coordinates scanner output at the workflow level only: ensure library, begin
  scan, record scan failures, commit directory observations, commit source
  observations, tombstone missing sources, and complete scan. The workflow
  Adapter owns Local Inference planning, confirmed/provisional item reuse,
  Source State, Library Item State, Local Inference Evidence, Search Projection
  planning, failure resolution, and the existing atomic scan-source persistence
  seam. Added a deletion-test style fake workflow test proving index callers do
  not require the broad repository trait set.
  Validation: `cargo check -p taru-library -p taru-db --tests`; `cargo nextest
  run -p taru-db scan_commit --no-fail-fast`; `cargo nextest run -p
  taru-library --no-fail-fast`; `cargo fmt --all -- --check`; `git diff
  --check`. PostgreSQL opt-in was not run because `TARU_TEST_POSTGRES_URL` was
  unset.
  Handoff: Continue with playback/transcode identity and diagnostics.

## M3 — Playback And Transcode Readiness

- [x] FAD-050 [owner=codex] [deps=FAD-040] [scope=crates/taru-streaming,crates/taru-transcode,crates/taru-server/src/app/playback,docs/workstreams/fearless-architecture-deepening]
  Goal: Define and test the Playback Source Selection + Transcode Profile
  request/cache identity before multi-profile HLS reuse, subtitles, HDR/SDR
  variants, or adaptive ladders widen the reuse surface.
  Validation: `cargo check -p taru-streaming -p taru-transcode -p taru-server
  --tests`; focused playback/profile identity nextest; `git diff --check`.
  Review: Do not add adaptive bitrate behavior in this task. The deliverable is
  a stable identity Interface and tests.
  Evidence: profile/request identity tests and docs.
  Progress: Added an explicit `PlaybackProfileIdentity` and a deeper
  `TranscodeRequestIdentity` that binds a `TranscodeProfileIdentity` to a
  `TranscodeSourceIdentity`. Remux and HLS session request keys and staging
  paths now use request identity instead of profile-only identity, so source
  revision changes, selected hardware policy, client capability/preferences,
  storage context, and transcode profile facts all participate in reuse/cache
  identity. Added tests for source-revision separation and hardware-profile
  separation without adding adaptive bitrate behavior.
  Validation: `cargo check -p taru-streaming -p taru-transcode -p taru-server
  --tests`; `cargo nextest run -p taru-streaming -p taru-transcode
  --no-fail-fast`; `cargo nextest run -p taru-server playback --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`.
  Handoff: Continue with hardware diagnostics.

- [x] FAD-060 [owner=codex] [deps=FAD-050] [scope=crates/taru-transcode,crates/taru-api,crates/taru-server/src/http/admin.rs,crates/taru-server/src/http/tests/system.rs,apps/admin-web/src/adminApi,docs/api,docs/workstreams/fearless-architecture-deepening]
  Goal: Deepen hardware acceleration diagnostics so static FFmpeg encoder
  discovery, device initialization evidence, and optional smoke-probe results
  are represented separately and reported safely.
  Validation: `cargo check -p taru-transcode -p taru-server --tests`; focused
  hardware diagnostics nextest; `git diff --check`.
  Review: Diagnostics must not require privileged devices in normal tests and
  must not leak local paths beyond safe operator diagnostics.
  Evidence: diagnostics model/tests and Admin diagnostics docs if surfaced.
  Progress: Replaced the single hardware evidence enum with separate
  `HardwareEncoderDiscovery`, `HardwareDeviceInitialization`, and
  `HardwareSmokeProbe` records. FFmpeg encoder discovery, optional device
  initialization, and optional smoke probes are now modeled independently;
  explicit device-init or smoke failures make an accelerator unavailable while
  normal startup tests remain unprivileged. Admin playback runtime diagnostics
  expose safe summaries for encoder discovery, device initialization, and smoke
  probes with detail booleans instead of raw probe text or device paths.
  Validation: `cargo check -p taru-transcode -p taru-api -p taru-server
  --tests`; `cargo nextest run -p taru-transcode --no-fail-fast`; `cargo
  nextest run -p taru-transcode hardware --no-fail-fast`; `cargo nextest run -p
  taru-api --lib admin_playback_runtime_diagnostics_serializes_safe_summary_fields
  --no-fail-fast`; `cargo nextest run -p taru-api --lib admin_contract
  --no-fail-fast`; `cargo nextest run -p taru-server
  admin_v1_playback_runtime_reports_safe_diagnostics --no-fail-fast`; `npm run
  check` in `apps/admin-web`; `cargo fmt --all -- --check`; `git diff
  --check`.
  Handoff: Continue with FAD-070 search semantics unless hardware diagnostics
  review finds an independent follow-on for real privileged smoke-probe
  execution.

## M4 — Search Semantics And Test Locality

- [x] FAD-070 [owner=codex] [deps=FAD-060] [scope=crates/taru-search,crates/taru-catalog,crates/taru-db,docs/workstreams/fearless-architecture-deepening]
  Goal: Add a small search semantics evaluation harness and projection-version
  discipline for title/alias/provider-title/CJK-friendly query behavior before
  AI or vector search is introduced.
  Validation: `cargo check -p taru-search -p taru-catalog -p taru-db --tests`;
  focused search nextest; `git diff --check`.
  Review: Do not add AI/vector search in this task. The deliverable is measured
  search semantics and future-safe projection discipline.
  Evidence: search evaluation fixtures/tests and documented semantics.
  Progress: Added `taru-search` shared query evaluation for title/alias/body/facet
  scoring, exact Browse Facet filtering, projection-version freshness helpers,
  and CJK-friendly compact matching. SQLite and PostgreSQL SearchIndex adapters
  now load search rows and delegate scoring/filtering to the shared evaluator
  instead of duplicating query semantics. Catalog hydration now projects accepted
  Provider Subject titles into Search Projection aliases/body/facets, so
  provider-title lookup is measured without adding AI/vector search.
  Validation: `cargo check -p taru-search -p taru-catalog -p taru-db --tests`;
  `cargo check -p taru-nfo -p taru-metadata -p taru-server --tests`; `cargo
  nextest run -p taru-search --no-fail-fast`; `cargo nextest run -p
  taru-catalog semantic_search --no-fail-fast`; `cargo nextest run -p taru-db
  search --no-fail-fast`; `cargo nextest run -p taru-db facet --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`. Some runs required
  setting `TMP`/`TEMP` to `F:\Temp` because `C:\Users\Frankorz\AppData\Local\Temp`
  had no free space.
  Handoff: Continue with FAD-080 test-locality cleanup for touched areas.

- [ ] FAD-080 [owner=codex] [deps=FAD-020,FAD-030,FAD-040,FAD-070] [scope=crates/taru-db/src/*tests*,crates/taru-server/src/http/tests,crates/taru-server/src/app/tests]
  Goal: Improve test locality around touched Interfaces by extracting
  domain-focused fixtures and splitting giant behavior families only where it
  improves reviewability.
  Validation: focused nextest for refactored test families; `cargo check
  --workspace --tests`; `git diff --check`.
  Review: Do not rewrite tests mechanically. Preserve coverage and failure
  meaning while reducing navigation cost.
  Evidence: smaller test Modules or shared fixtures around changed Interfaces.
  Handoff: Continue with final closeout or split remaining tails.

## M5 — Closeout Or Split

- [ ] FAD-090 [owner=planner] [deps=FAD-080] [scope=docs/workstreams/fearless-architecture-deepening,docs/GOALS.md,docs/ROADMAP.md,docs/workstreams/README.md]
  Goal: Verify the fearless refactor lane, close it, and split any remaining
  independent tails into named workstreams.
  Validation: `cargo fmt --all -- --check`; `cargo check --workspace --tests`;
  `cargo nextest run --workspace --no-fail-fast`; PostgreSQL opt-in contracts
  for touched persistence seams when `TARU_TEST_POSTGRES_URL` is available;
  `git diff --check`.
  Review: Use review/verify workstream discipline before marking the goal
  complete. Remaining work must not hide inside vague "follow up" text.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Recommend the next product lane only after architecture debt is
  either closed or explicitly split.
