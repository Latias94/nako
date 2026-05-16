# Server Foundation Milestones

## M0: Architecture Baseline

Outcome: the repository has a Rust workspace skeleton and documented module
boundaries.

Deliverables:

- Workspace manifest and initial crate stubs.
- Shared error, ID, and configuration model.
- ADRs for architecture, VFS, addons, and automation boundary.
- Basic CI commands documented.

Exit criteria:

- `cargo fmt` succeeds.
- `cargo nextest run` or `cargo test` succeeds for available crates.
- Crate dependency direction is documented.

## M1: Local Library Index

Outcome: Taru can index a local media directory into a persistent database.

Deliverables:

- Local filesystem VFS backend.
- Library scan job.
- Naming parser MVP for movies and series.
- SQLite schema for libraries, items, media sources, streams, and external IDs.
- ffprobe integration behind `taru-media-probe`.

Exit criteria:

- A test fixture library can be scanned repeatedly without duplicate items.
- File changes are detected by rescan.
- Probe data is stored and exposed through an internal API.

## M2: Server Runtime and API Foundation

Outcome: Taru can run as a long-lived local server with a minimal HTTP API and
persisted background job state.

Deliverables:

- HTTP server bootstrap in `taru-server`.
- Startup configuration for listen address, SQLite, ffprobe, local library, and
  scan/probe concurrency.
- Persisted job table and repository.
- Background library scan job triggered through HTTP.
- Minimal API for health, libraries, sources, items, probes, and jobs.
- Structured logging and safe JSON error responses.
- CLI scan/list commands sharing the same application service.

Exit criteria:

- `POST /libraries/{id}/scan` returns a queued job without blocking.
- `GET /jobs/{id}` reports queued, running, succeeded, or failed state.
- Scan and probe work remains bounded by configured limits.
- `cargo fmt`, `cargo check`, and `cargo nextest run` pass for the workspace.

## M2.1: Runtime Hardening and API Discipline

Outcome: Taru's server runtime has durable job inputs, documented job
semantics, pagination basics, and developer workflow documentation.

Deliverables:

- Persisted `input_json` for jobs.
- Job lifecycle ADR covering cancellation, retry, idempotency, failure
  isolation, and resource budgets.
- Limit/offset pagination for current list routes.
- API envelope documentation for errors, jobs, and pagination.
- Local development guide.
- Test strategy.
- Licensing and GPL reference-code boundary notes.

Exit criteria:

- Job input is persisted in SQLite and returned by `GET /jobs/{id}`.
- `GET /libraries`, `GET /libraries/{id}/sources`, and `GET /items` accept
  `limit` and `offset`.
- Invalid pagination returns `400`.
- `cargo fmt`, `cargo check`, and `cargo nextest run` pass for the workspace.

## M3 Summary: Metadata and NFO

Outcome: Taru can enrich indexed items and preserve local metadata control.

Deliverables:

- TMDB provider MVP.
- NFO import/export MVP.
- Provider priority, field lock, and external ID model.
- Raw provider response cache.

Exit criteria:

- A movie and a series fixture can be matched and enriched.
- Locked local fields survive metadata refresh.
- NFO export round-trips core fields.

## M3.1: Metadata Model and NFO Policy Foundation

Outcome: Taru has a provider-neutral metadata model, merge policy, raw provider
cache, and minimal movie NFO codec.

Deliverables:

- Expanded canonical metadata model.
- Image, rating, genre, and credit primitives.
- Field-level metadata locks.
- Provider raw response cache.
- Minimal movie NFO import/export codec.
- ADRs for metadata authority and NFO local boundary.

Exit criteria:

- Locked fields survive metadata merge.
- Rich metadata round-trips through SQLite.
- Provider raw cache round-trips through SQLite.
- Movie NFO core fields round-trip through `taru-nfo`.
- `cargo fmt`, `cargo check`, and `cargo nextest run` pass for the workspace.

## M3.2: TMDB Provider MVP and Metadata Refresh Job

Outcome: Taru can enrich an indexed movie item through TMDB while preserving
local metadata authority.

Deliverables:

- TMDB movie search and movie-detail provider implementation.
- Provider trait with search and fetch paths.
- Metadata refresh service using field locks and merge policy.
- Persisted metadata refresh jobs with durable inputs and summaries.
- Raw TMDB detail response cache.
- Configured provider secret references resolved from environment variables.
- HTTP and CLI triggers for refreshing one item.

Exit criteria:

- Refresh uses an existing TMDB external ID without search.
- Refresh can search by title/year, fetch details, and merge canonical metadata.
- Locked fields survive refresh.
- Raw TMDB detail responses are stored in SQLite.
- Job inputs do not include provider secrets.
- Tests use mocked provider responses and do not require real TMDB network calls.
- `cargo fmt`, `cargo check`, and `cargo nextest run` pass for the workspace.

## M3.3: Library Profiles and Metadata Strategy

Outcome: Taru can choose metadata resolution behavior from library-level
profiles instead of hard-coding provider behavior into refresh endpoints.

Deliverables:

- ADR for treating library presets as editable configuration templates.
- Library domain and preset model.
- Library options model for scan, naming, local metadata, and refresh behavior.
- Metadata profile model for local readers, provider order, image providers,
  language, country, refresh mode, and local authority policy.
- SQLite persistence for library options and metadata profiles.
- Metadata refresh planning based on the effective library profile.
- Preset defaults for movies, TV, anime, music, podcast, photos, home video,
  mixed video, and future online catalogs.

Exit criteria:

- A library can store domain, preset, and metadata profile options.
- Preset defaults can be generated and then edited without changing the core
  item type model.
- Metadata refresh resolves provider order from the library profile.
- Disabled providers are skipped.
- `missing_only` fills empty unlocked fields without replacing populated
  values.
- `full_refresh` updates unlocked fields while preserving locked fields.
- `cargo fmt`, `cargo check`, and `cargo nextest run` pass for the workspace.

## M3.4: Metadata Strategy Executor and Provider Fallback

Outcome: Taru can execute metadata refresh through a provider registry and
profile-ordered fallback strategy instead of server-side provider branching.

Deliverables:

- Metadata provider registry with available, disabled, and unavailable states.
- Metadata strategy executor that tries providers in profile order.
- Refresh summaries containing attempted providers and selected provider.
- Server integration that builds the provider registry from runtime config.
- Fallback behavior for unimplemented, disabled, unavailable, no-match, and
  provider-failure outcomes.

Exit criteria:

- Bangumi not implemented can fall back to TMDB when TMDB is registered.
- Disabled and unavailable providers are skipped with attempt summaries.
- All providers failing produces a failed metadata refresh job.
- First successful provider short-circuits later providers.
- Field locks, `missing_only`, and `full_refresh` behavior remain intact.
- Server no longer rejects non-TMDB first providers with hard-coded logic.
- `cargo fmt`, `cargo check`, and `cargo nextest run` pass for the workspace.

## M3.5: NFO Discovery, Import, and Export Jobs

Outcome: Taru can discover same-stem NFO sidecars, import them as local
metadata, and export canonical metadata through persisted jobs.

Deliverables:

- VFS text read/write methods for sidecar files.
- Same-stem NFO sidecar discovery.
- NFO import service using local metadata policy.
- NFO export service using `write_sidecar` policy.
- NFO import and export job kinds, summaries, HTTP routes, and CLI commands.
- Field locks written with `MetadataSource::Nfo` when NFO is local authority.

Exit criteria:

- `local_first` and `read_only` imports update unlocked fields and create NFO
  locks.
- `remote_first` imports only fill missing fields and do not create NFO locks.
- User-locked fields survive NFO import.
- `write_sidecar` exports movie NFO sidecars.
- HTTP routes queue NFO import/export jobs.
- `cargo fmt`, `cargo check`, and `cargo nextest run` pass for the workspace.

## M3.6: Catalog Graph, Artwork Cache, Search, and Scan Strategy

Outcome: Taru has an implementation plan for the catalog graph and performance
foundations needed by actor/director pages, tag filters, image-heavy UI,
search, and incremental scanning.

Deliverables:

- Catalog graph design for people, credits, tags, genres, collections, studios,
  images, and external IDs.
- Artwork cache and preview-generation performance strategy.
- Search-index strategy covering embedded and optional external adapters.
- Incremental scan and remote-storage scan strategy.
- Updated roadmap and TODO tracking for these foundations.

Exit criteria:

- Actor/director relationship modeling is documented.
- Tags, genres, collections, studios, and user/local authority boundaries are
  documented.
- Artwork list-page performance rules and resource classes are documented.
- Search indexing sources, update triggers, and adapter boundaries are
  documented.
- Incremental scanning, directory cache, tombstones, rename detection, and
  remote listing policies are documented.
- The workstream README points to the current design focus.

## M4.0: Catalog Ingestion Foundation

Outcome: Taru can run the first server-side library ingestion loop from local
VFS scan to persisted media items, scan state, normalized catalog graph,
search projection, and artwork task queue records.

Deliverables:

- Catalog graph domain models and SQLite tables for people, credits, genres,
  tags, collections, studios, and image assets.
- Durable scan snapshots, directory snapshots, source states, and tombstones.
- Local VFS fingerprints based on lightweight file metadata.
- SQLite search projection behind the `taru-search` adapter boundary.
- HTTP search route for projected items.
- Artwork task table with resource classes, retry state, and default concurrency
  policy.
- ADRs for catalog/search projection, scan state, and artwork resource classes.

Exit criteria:

- A local directory scan creates a library, media items, sources, scan state,
  source states, and search documents.
- A disappeared source becomes tombstoned on a later scan.
- People, credits, tags, genres, collections, studios, and image assets
  round-trip through SQLite.
- Search can return projected items through the HTTP route.
- Artwork preview/fetch/resize work can be persisted with retry state and
  resource class.
- `cargo fmt`, `cargo check`, and `cargo nextest run` pass for the workspace.

## M4.1: Catalog Graph Hydration and Browse API

Outcome: Taru hydrates the normalized catalog graph from metadata refresh and
NFO import, rebuilds search projection after metadata changes, and exposes the
first browse API surface needed by future clients.

Deliverables:

- Shared `taru-catalog` hydration module from canonical metadata to graph
  records.
- Metadata refresh graph hydration for people, credits, genres, tags,
  collections, studios, image assets, and search projection.
- NFO import graph hydration for genres, tags, actors, directors, writers,
  local artwork references, and search projection.
- SQLite repository methods for natural-key graph lookup and item reverse
  lookup by person, tag, and genre.
- Browse HTTP routes for item detail, item credits/images, people, tags, and
  genres.

Exit criteria:

- TMDB refresh writes canonical metadata and normalized graph records.
- NFO import writes canonical metadata and normalized graph records.
- Search can find metadata-derived people, tags, and genres after refresh or
  NFO import.
- Browse routes return graph-linked items through paginated list endpoints.
- `cargo fmt`, `cargo check`, and `cargo nextest run` pass for the workspace.

## M4.2: Playback Decision and Direct Play API

Outcome: Taru can decide whether a source can direct play and serve local media
through an HTTP byte-range direct play route.

Deliverables:

- Playback decision model.
- Source-level playback decision route.
- Direct play route with HTTP `Range` support.
- MIME/container inference for common local video containers.
- Source lookup by ID for playback/probe/API paths.
- Tests for direct play decisions and partial-content streaming.

Exit criteria:

- Compatible local files direct play without FFmpeg.
- `Range: bytes=start-end` returns `206 Partial Content` with correct
  `Content-Range`, `Content-Length`, and body bytes.
- Unsupported containers/codecs produce remux or transcode decisions without
  starting FFmpeg.
- `cargo fmt`, `cargo check`, and `cargo nextest run` pass for the workspace.

## M4.2.1: Direct Play Boundary Hardening

Outcome: Taru has a direct play boundary that is ready for remux, HLS, and
transcode work without keeping response policy in raw HTTP handlers.

Deliverables:

- Direct play response planning model in `taru-streaming`.
- Application-level direct play planning in `taru-server::app`.
- HTTP handler reduced to header translation, file streaming, and response
  mapping.
- `HEAD /sources/{source_id}/stream` support for playback preflight.
- Edge-case tests for zero-byte files, invalid ranges, unsatisfiable ranges,
  and unsupported multi-range requests.
- Updated API and workstream documentation.

Exit criteria:

- Direct play response plans cover `200 OK`, `206 Partial Content`, and
  `416 Range Not Satisfiable`.
- `HEAD /sources/{source_id}/stream` returns direct play headers without a
  body.
- `416` direct play responses include `Content-Range: bytes */{total_len}`.
- `cargo fmt`, `cargo check`, `cargo nextest run`, and `git diff --check`
  pass for the workspace.

## M4.3: FFmpeg Command Builder and Remux Session Skeleton

Outcome: Taru has an explicit FFmpeg planning boundary and a remux session
skeleton before any process runner, HLS serving, or hardware acceleration is
implemented.

Deliverables:

- FFmpeg command builder interface in `taru-transcode`.
- Copy-only remux command planning for MP4 and Matroska outputs.
- Remux request validation for empty and in-place paths.
- In-memory transcode session model and lifecycle transition checks.
- `ffmpeg_path` server configuration with default and config tests.
- Workstream documentation for the remux skeleton and its non-goals.

Exit criteria:

- Remux planning produces `-map 0 -c copy` without spawning FFmpeg.
- Invalid remux requests fail before a command plan is returned.
- Session transitions reject invalid lifecycle moves.
- `cargo fmt`, `cargo check`, `cargo nextest run`, and `git diff --check`
  pass for the workspace.

## M4.4: Remux Process Runner and Runtime Resource Guard

Outcome: Taru can run a planned remux session through a real FFmpeg process
behind explicit lifecycle, cancellation, timeout, concurrency, and cleanup
boundaries without exposing a playback HTTP route yet.

Deliverables:

- FFmpeg remux process runner in `taru-transcode`.
- Cancellation token and timeout handling for remux sessions.
- Semaphore-backed remux runtime guard.
- Session-scoped temporary output path with promote-on-success behavior.
- Temporary output cleanup on failure, cancellation, and timeout.
- Server runtime config for remux concurrency and timeout.
- Workstream documentation for runner behavior and non-goals.

Exit criteria:

- Runner promotes temporary output only after FFmpeg exits successfully.
- Runner kills the child process and cleans temporary output on cancellation
  and timeout.
- Failed FFmpeg exits mark the session failed and leave no temporary output.
- Runtime guard prevents more than the configured concurrent remux sessions.
- No HTTP remux/playback route is exposed in this phase.
- `cargo fmt`, `cargo check`, `cargo nextest run`, and `git diff --check`
  pass for the workspace.

## M4.5: Remux App Service Integration and Local Staging Policy

Outcome: Taru has an application-level remux orchestration boundary and a local
staging policy that can support a future HTTP remux playback route without
letting HTTP handlers own FFmpeg process details.

Deliverables:

- Remux application service in `taru-server::app`.
- Local remux staging directory policy.
- Deterministic output naming and cleanup rules.
- Duplicate remux request reuse or explicit idempotency behavior.
- API-safe error mapping for invalid requests, runner failures,
  cancellation, and timeouts.
- Tests using fake runners or command planning where possible.
- Workstream documentation for app-service behavior and non-goals.

Exit criteria:

- Server composition can create and call the remux app service.
- Staging paths cannot escape the configured staging root.
- Duplicate remux requests have deterministic behavior.
- Runner failures map to stable application errors without leaking process
  internals.
- No public HTTP remux/playback route is exposed in this phase.
- `cargo fmt`, `cargo check`, `cargo nextest run`, and `git diff --check`
  pass for the workspace.

## M4.6: Remux Playback Route

Outcome: Taru exposes remuxed playback through HTTP while keeping handlers as
request/response translation around the remux application service.

Deliverables:

- HTTP remux playback route.
- Client capability parsing for remux decisions.
- Streaming response for completed staged remux outputs.
- Conflict or pending response for equivalent in-flight remux requests.
- API documentation for the route.
- Handler tests proving the route does not bypass the app service.

Exit criteria:

- A local source with compatible codecs and unsupported container can be remuxed
  and streamed through HTTP.
- Completed staged outputs are reused.
- In-flight duplicate remux requests return a stable conflict response.
- Direct play route behavior remains unchanged.
- No HLS, remote staging, or hardware acceleration route is exposed in this
  phase.
- `cargo fmt`, `cargo check`, `cargo nextest run`, and `git diff --check`
  pass for the workspace.

## M4.7: Playback Session Persistence

Outcome: remux and future transcode sessions are persisted so cancellation,
retry, observability, and multi-client playback can survive process restarts.

Deliverables:

- SQLite schema for transcode/remux sessions.
- Repository and domain model for session state, request key, source ID,
  output path, kind, timestamps, and failure message.
- App service integration that records planned, running, finished, failed, and
  cancelled session states.
- HTTP or app-level lookup path for current session state.
- Cleanup and retry policy for stale or failed sessions.

Exit criteria:

- A remux route creates or reuses a persisted session record.
- Session state transitions survive process restart through SQLite.
- Duplicate request behavior uses persisted state rather than only in-memory
  request keys.
- Failed sessions retain safe error categories and diagnostic detail for logs.
- `cargo fmt`, `cargo check`, `cargo nextest run`, and `git diff --check`
  pass for the workspace.

## M4.8: HLS Transcode Foundation

Outcome: Taru can model and serve the first HLS playlist/segment lifecycle on
top of persisted playback sessions before hardware acceleration policy is
introduced.

Deliverables:

- HLS output layout and staging policy.
- FFmpeg HLS command planning for a minimal single-variant stream.
- HLS app-service boundary using persisted transcode sessions.
- HTTP routes for playlists and segment bytes.
- Duplicate request, stale session, and failure handling aligned with remux
  session persistence.

Exit criteria:

- HLS session creation persists a playback session.
- Playlist and segment paths cannot escape the staging root.
- Active equivalent HLS requests return deterministic conflict or reuse
  behavior.
- Finished sessions can be reused when artifacts still exist.
- Failed HLS runs persist a safe failure category/message.
- `cargo fmt`, `cargo check`, `cargo nextest run`, and `git diff --check`
  pass for the workspace.

## M4.9: Hardware Acceleration Policy

Outcome: Taru has a deterministic hardware acceleration capability and policy
model before VAAPI, NVENC, or QuickSync are enabled in transcode command
execution.

Deliverables:

- Hardware acceleration domain model for CPU-only, VAAPI, NVENC, and QuickSync.
- Detection boundary for available accelerators.
- Encode policy model that can choose CPU-only or a declared accelerator.
- CPU/GPU queue and resource-budget defaults.
- Stable fallback/error behavior for unavailable accelerators.
- Tests that do not require real GPU hardware.

Exit criteria:

- Capability and policy records can be planned without starting FFmpeg.
- Unsupported or unavailable accelerators produce stable outcomes.
- HLS command planning can consume a policy decision without HTTP handlers
  knowing vendor-specific details.
- Queue/resource defaults are bounded and documented.
- `cargo fmt`, `cargo check`, `cargo nextest run`, and `git diff --check`
  pass for the workspace.

## M4.10: MVP Stabilization

Outcome: the local video-library playback MVP is documented, bounded, and
validated before broader extension or remote-storage work begins.

Deliverables:

- HTTP API audit and documentation updates.
- Config documentation for scan, metadata, remux, HLS, hardware policy, and
  resource budgets.
- Error mapping audit for playback/session/job routes.
- Focused edge-case tests for gaps found during the audit.
- Known limitations and MVP non-goals.

Exit criteria:

- API docs match implemented routes.
- Config docs match implemented fields and defaults.
- Validation gates pass for the workspace.
- Any remaining MVP limitation is explicitly documented.
- `cargo fmt`, `cargo check`, `cargo nextest run`, and `git diff --check`
  pass for the workspace.

## M5: Extension Surface

Outcome: Taru has stable external automation and addon surfaces.

Detailed M5 milestones are tracked in
`docs/workstreams/addons-automation/MILESTONES.md`.

Deliverables:

- Webhook outbox.
- Automation job runner.
- External API provider configuration.
- Taru addon manifest draft and one reference addon.

Exit criteria:

- Library scan and metadata events can trigger webhooks.
- An automation task can call a configured external provider.
- A reference HTTP addon can return metadata or recommendations.

## M6: Remote Storage Preview

Outcome: Taru can index and play a limited remote source through the internal
VFS contract.

Deliverables:

- WebDAV or S3-compatible backend preview.
- Directory cache.
- Byte-range cache or local staging path.
- Remote-source playback policy.

Exit criteria:

- A remote fixture can be scanned without treating it as a local path.
- Probe/transcode can read remote media through cache or staging.
- Rate limits and retry behavior are configurable.

## M20: Server Surface Decomposition

Outcome: `taru-server` test and route surfaces are organized by bounded
context, so future playback, metadata, storage, addon, automation, and catalog
changes can be reviewed without editing giant shared test files.

Deliverables:

- Split `crates/taru-server/src/app/tests.rs` into `app/tests/{startup,
  storage,nfo,metadata,playback,staging}.rs`.
- Split `crates/taru-server/src/http/tests.rs` into `http/tests/{system,
  webhooks,automation,addons,library,metadata,catalog,playback}.rs`.
- Keep shared fixtures in each test directory's `mod.rs`.
- Avoid route or application behavior changes.
- Preserve explicit API DTO usage in HTTP tests.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace`
- `git diff --check`

## M21: Storage Backend Registry

Outcome: storage backends are owned by a library-scoped runtime registry instead
of being constructed ad hoc by scan, playback, NFO, or staging call sites.

Deliverables:

- `TaruApp` owns a `StorageBackendRegistry` keyed by configured `library_id`.
- `LibraryStorageBackend` wraps concrete local/WebDAV VFS backends and owns
  per-library runtime state.
- Direct-play stream permits, remote staging permits, WebDAV cache state, and
  backend health counters are shared through the registry-owned backend.
- Scan/probe/playback/NFO/remux/HLS storage access resolves through the app
  registry helpers.
- Media sources resolve backends by `source.library_id`; URI fallback matching
  is removed.
- `/storage/backends` exposes safe process-local registry diagnostics.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check -p taru-server --tests`
- `cargo nextest run -p taru-server`
- `git diff --check`

## M22: Ingestion Failure Diagnostics

Outcome: scan and probe failures are isolated, durable, and diagnosable without
turning partial local or remote ingestion failures into opaque whole-library
failures.

Deliverables:

- Add an `ingestion_failures` persistence model for scan and probe failures.
- Record scan directory/object failures without blocking successful sibling
  media sources.
- Record per-source probe/open/stage/ffprobe failures without blocking other
  sources.
- Keep partial scan failures from tombstoning existing sources from incomplete
  passes.
- Add retry/resolve semantics through repeated successful ingestion and ignore
  semantics for expected failures.
- Expose HTTP and CLI diagnostics for open, resolved, and ignored failures.
- Include scan/probe failure counts in job summaries and library scanned events.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace`
- `git diff --check`

## M23: API, HTTP, and DB Boundary Cleanup

Outcome: the public API and server composition layers keep explicit DTO and
bounded-context boundaries so future metadata, NFO, automation, and client
changes can be made without re-entangling domain records with transport shapes
or root-level router tables.

Deliverables:

- Explicit DTOs for high-frequency library, catalog, playback, source probe,
  transcode session, and ingestion failure responses.
- HTTP route registration moved into bounded-context router functions.
- Remaining `taru-db/src/lib.rs` helper methods moved into their owning
  repository modules.
- DTO serialization coverage and API error-body coverage for the cleaned-up
  surfaces.
- Updated workstream docs and future API constraint notes.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace`
- `git diff --check`
