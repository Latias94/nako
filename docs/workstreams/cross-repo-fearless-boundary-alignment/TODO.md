# Cross-Repo Fearless Boundary Alignment - TODO

Status: Active
Last updated: 2026-05-25

Task IDs use the `CRFBA` prefix.

## M0 - Scope And Evidence Freeze

- [x] CRFBA-010 [owner=planner] [deps=none] [scope=docs/workstreams/cross-repo-fearless-boundary-alignment]
  Goal: Confirm the cross-repo target state, non-goals, reference-product
  lessons, and no-code-before-scope-freeze rule with the user.
  Validation: `DESIGN.md`, `TODO.md`, `MILESTONES.md`,
  `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and `HANDOFF.md` agree.
  Review: Confirm this lane does not duplicate completed Nako or official addon
  workstreams.
  Evidence: this workstream directory.
  Result: DONE 2026-05-24. User accepted the fearless refactor scope and asked
  to start executing all listed architecture work.
  Handoff: Do not begin implementation refactors until this task is accepted.

- [x] CRFBA-011 [owner=planner] [deps=CRFBA-010] [scope=F:/SourceCodes/Rust/nako,F:/SourceCodes/Rust/nako-official-addons]
  Goal: Record fresh git status, active workstreams, and safe-edit constraints
  for both repositories before any worker starts.
  Validation: status notes in `HANDOFF.md`; unrelated dirty files are listed as
  protected.
  Review: No worker may format, restore, delete, or stage unrelated files.
  Evidence: `HANDOFF.md`.
  Result: DONE 2026-05-24. Dirty worktree notes are recorded in `HANDOFF.md`;
  `crates/nako-library` had no local modifications before the first code slice.
  Handoff: Split tasks by repository when dirty worktrees make parallel edits
  risky.

## M1 - Server Workflow Port Deepening

- [ ] CRFBA-020 [owner=unassigned] [deps=CRFBA-011] [scope=crates/nako-db,crates/nako-core,crates/nako-server]
  Goal: Choose one high-leverage server workflow and replace broad database
  access at the call site with a workflow-shaped port without breaking
  SQLite/PostgreSQL parity.
  Progress: `crates/nako-server/src/app/addons.rs` now routes addon
  registration and grant operations through a dedicated narrow registration
  store, and `crates/nako-server/src/app/acquisition_intake.rs` now routes
  candidate record/list/discovery/acceptance through a dedicated workflow
  store instead of reaching into a raw `NakoDatabase` handle.
  `crates/nako-server/src/app/job_runtime.rs` now routes durable job lease
  claim/heartbeat/succeed/fail/cancel through a dedicated
  `DurableJobLeaseStore` instead of a universal database handle.
  `crates/nako-server/src/app/metadata.rs` now routes direct metadata refresh,
  maintenance, raw-response, and attempt queries through a dedicated
  `MetadataWorkflowStore` instead of reaching straight into `NakoDatabase` for
  those read/write calls.
  `crates/nako-server/src/app/nfo.rs` now routes NFO job creation,
  library/item/source lookups, sidecar-apply audit state transitions, outbox
  writes, and durable job runtime lease handoff through a dedicated
  `NfoWorkflowStore`; `NfoService` keeps its own repository dependency for
  import/export domain work.
  `crates/nako-server/src/app/jobs.rs` now routes library scan enqueue,
  library lookup, outbox writes, scan ingestion, probe execution, and
  ingestion-failure bookkeeping through `LibraryScanWorkflowStore` and a
  dedicated execution store instead of a raw `NakoDatabase` handle.
  `crates/nako-server/src/app/playback/mod.rs` now routes playback decisions,
  remux/HLS execution, transcode-session access, cancellation, and
  finished-event writes through `PlaybackRuntimeStore`.
  `crates/nako-server/src/app/playback/input.rs` now routes staging-record
  lookup and lease acquisition through `Arc<dyn StagingManifestRepository>`,
  and `PlaybackAppService::new` accepts runtime/staging ports instead of a raw
  `NakoDatabase` handle.
  `crates/nako-library/src/probe.rs` now depends on `LibraryProbeWorkflow`
  so probe execution only requires source/probe/failure operations instead of
  the full repository set.
  `crates/nako-metadata/src/lib.rs` now re-exports focused metadata strategy
  ports so `MetadataExecutionStore` can depend on refresh snapshot/commit and
  attempt-record operations without widening the public metadata surface.
  Validation: focused nextest for the touched workflow and backend contract
  tests; targeted `nako-library`, `nako-metadata`, `nako-server metadata`,
  `nako-server playback`, `nako-server staging`, and `nako-server startup`
  gates; `cargo check -p nako-server`; `cargo fmt --all -- --check` when
  practical.
  Review: The slice must reduce caller authority, not just rename repository
  traits.
  Evidence: focused tests and updated architecture notes.
  Handoff: If the first workflow is too broad, split a smaller persistence-port
  proof before touching `NakoDatabase` structure.

- [x] CRFBA-030 [owner=codex] [deps=CRFBA-011] [scope=crates/nako-library]
  Goal: Extract Local Inference planning from Library scan persistence commit
  so ingestion can evolve naming rules, duplicate evidence, and search
  projection independently.
  Progress: `crates/nako-library/src/ingestion/source_commit.rs` now owns the
  source-observation planning path; `ingestion.rs` only orchestrates the
  workflow and maps the disposition back to the public summary.
  Validation: `cargo nextest run -p nako-library` plus focused `nako-db`
  repository/contract tests when persistence behavior changes.
  Review: The ingestion workflow should read as "observe source, plan local
  evidence, commit planned mutation" rather than one mixed commit method.
  Evidence: tests covering unchanged scan/source/tombstone behavior.
  Handoff: Preserve existing scan semantics before broadening inference rules.
  Result: DONE_WITH_CONCERNS 2026-05-24. Focused source-observation and exact
  `local_inference::tests::*` targets pass; the broader package filter is still
  blocked by an unrelated migration 38 duplicate-column conflict outside this
  slice.

- [ ] CRFBA-040 [owner=codex] [deps=CRFBA-020] [scope=crates/nako-metadata,crates/nako-nfo,crates/nako-catalog,crates/nako-server]
  Goal: Make Candidate/Acceptance vocabulary the explicit host-side write
  boundary for provider refresh, NFO import/export, Addon protected writes,
  artwork selection, and future AI-like suggestions.
  Progress: `crates/nako-server/src/app/artwork.rs` now routes candidate
  lookup, media/item-state validation, and acceptance commit through a
  dedicated `ArtworkAcceptanceWorkflowStore` instead of calling the broad
  `NakoDatabase` facade directly. It also now routes publish/select/unpublish
  through a dedicated `ArtworkSelectionWorkflowStore` instead of calling the
  broad store directly for selection writes. It also routes ingest
  claim/requeue/commit/fail through a dedicated `ArtworkIngestWorkflowStore`
  instead of calling the broad store directly for ingest writes.
  Validation: focused tests for one accepted metadata or artwork path; docs
  updated if public semantics change.
  Review: Providers and Addons must produce evidence-backed candidates, not
  hidden canonical commits.
  Evidence: candidate/acceptance tests and updated ADR/workstream notes.
  Handoff: Split broad product behavior into follow-ons instead of widening this
  slice.

## M2 - Official Addon Runtime Alignment

- [x] CRFBA-050 [owner=codex] [deps=CRFBA-011] [scope=../nako-official-addons/crates/nako-metadata-scraper/src/engine]
  Goal: Split `MetadataScrapeRuntime` into deeper internal modules for request
  decoding, provider orchestration, candidate selection, writeback
  coordination, and response/artifact shaping while preserving public payloads.
  Validation: `cargo nextest run -p nako-metadata-scraper metadata writeback artwork ranking --no-fail-fast`.
  Review: Route behavior and protocol payload shape must remain compatible.
  Evidence: runtime tests before/after the split.
  Result: DONE 2026-05-24. `engine` now splits request/query handling,
  provider fan-out, response shaping, writeback coordination, and bulk task
  planning across `query.rs`, `orchestration.rs`, `response.rs`,
  `runtime.rs`, `writeback.rs`, and `bulk.rs` without changing public payloads.
  Handoff: Continue with CRFBA-060 for the first provider adapter split.

- [x] CRFBA-060 [owner=codex] [deps=CRFBA-050] [scope=../nako-official-addons/crates/nako-metadata-scraper/src/providers]
  Goal: Split one large provider adapter first, preferably TMDB, into
  provider-local client/search/enrichment/mapper/parser/test-support modules.
  Validation: focused provider tests for direct ID lookup, title search,
  enrichment, degraded candidates, ranking inputs, and malformed payload
  resilience.
  Review: The split must preserve provider behavior and improve locality for
  future provider changes.
  Evidence: focused tests and module map in handoff notes.
  Result: DONE 2026-05-24. `providers::tmdb` now uses dedicated `client`,
  `search`, `parser`, `mapper`, `enrichment`, and `test_support` modules while
  keeping public provider behavior unchanged. Focused TMDB provider tests pass
  against direct lookup, search, enrichment, degraded candidates, ranking, and
  malformed payload cases.
  Handoff: Repeat for Bangumi and Douban only after the first provider split is
  reviewed.

- [x] CRFBA-070 [owner=codex] [deps=CRFBA-050] [scope=crates/nako-addon-client,crates/nako-addon-protocol,../nako-official-addons/crates/nako-metadata-scraper/src/nako_runtime.rs]
  Goal: Align official addon protected-write host-client responsibilities with
  public protocol/client crates without depending on private server crates.
  Progress: `crates/nako-addon-protocol` now exposes public Addon permission,
  access-check, side-effect target, side-effect request/response, and typed
  metadata/artwork write request shapes. `crates/nako-addon-client` now owns the
  reusable Nako Runtime HTTP client behavior for access-check and protected
  side-effect submission, including bearer-token header placement, token-body
  leak rejection, retryability classification, redaction-safe HTTP errors, and
  version-tolerant side-effect response parsing. The official metadata scraper
  now keeps a thin `nako_runtime` facade over the public client/protocol crates
  instead of carrying its own duplicated request, transport, and error
  implementation.
  Validation: protocol/client tests plus addon fake-transport tests proving
  bearer token placement, redaction, safe error mapping, and version tolerance.
  Review: Do not move reqwest-heavy runtime behavior into the permissive
  protocol crate unless an ADR says so.
  Evidence: tests and any required ADR/workstream update.
  Result: DONE_WITH_CONCERNS 2026-05-24. Focused protocol/client and official
  addon runtime facade tests pass, including a regression test proving reqwest
  transport errors do not expose request URLs or query tokens. The official
  addon currently uses local path dependencies to the public client/protocol
  crates for this cross-repo proof; public crate release/publishing remains a
  separate follow-on if distribution needs it.
  Handoff: Review this cross-repo contract slice before continuing to
  CRFBA-080/090. If a public crate release is needed, split release/versioning
  into a separate lane.

## M3 - Playback Runtime Ownership

- [x] CRFBA-080 [owner=codex] [deps=CRFBA-011] [scope=crates/nako-streaming,crates/nako-transcode,crates/nako-server,crates/nako-api]
  Goal: Clarify and deepen ownership between Playback Runtime and transcode
  execution so session lifecycle, cancellation, failure taxonomy, hardware
  diagnostics, and operator-visible state have one product owner.
  Validation: focused playback/transcode nextest gates; diagnostics contract
  tests if API output changes.
  Review: `nako-transcode` should expose execution APIs; Playback Runtime
  should own product decisions and session state.
  Evidence: tests plus architecture notes.
  Result: DONE 2026-05-29. `nako-transcode` now exposes
  `TranscodeExecutionRequest` and typed engine outcomes instead of a public
  product-state `TranscodeSessionManager`. The FFmpeg HLS/Remux runners now
  execute immutable requests and return output/metrics/cancellation outcomes;
  `Playback Runtime` in `nako-server` owns persisted transcode-session state,
  cancellation registration, failure taxonomy, finished events, and HLS
  metrics persistence.
  Handoff: Split adaptive bitrate, optimized versions, or new hardware backend
  breadth into follow-ons.

## M4 - Contract And Closeout

- [x] CRFBA-090 [owner=codex] [deps=CRFBA-040,CRFBA-070] [scope=crates/nako-api,crates/nako-server,crates/nako-addon-protocol]
  Goal: Decide whether touched API/protocol surfaces need generated contract
  support or stronger route/schema registries to prevent drift.
  Validation: existing OpenAPI/admin contract tests plus focused tests for any
  touched surface.
  Review: This is a hardening slice, not permission to redesign every route.
  Evidence: contract tests and updated docs.
  Result: DONE 2026-05-25. The addon runtime routes now have a public protocol
  route inventory, the Nako Runtime client and server route registration use
  the same path constants, Public Client OpenAPI exclusion checks iterate the
  addon runtime inventory, and `nako-api` has parity tests proving
  access-check and side-effect DTOs serialize to the public protocol wire
  shape.
  Handoff: No generated Addon Runtime SDK was added. The current hardening
  need is covered by route inventory plus wire-shape parity tests; generated
  client/schema output remains a separate follow-on if external Addon authors
  need published artifacts beyond the Rust protocol/client crates.

- [ ] CRFBA-100 [owner=planner] [deps=CRFBA-020,CRFBA-030,CRFBA-040,CRFBA-050,CRFBA-060,CRFBA-070,CRFBA-080] [scope=docs/workstreams/cross-repo-fearless-boundary-alignment]
  Goal: Review the lane, record final gates, update status, and split remaining
  product breadth into named follow-ons.
  Validation: `verify-rust-workstream` records fresh evidence; `review-workstream`
  has no blocking findings.
  Review: Close only when shipped behavior and docs match.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`.
  Handoff: Remaining work must be explicitly completed, deferred, or split.
