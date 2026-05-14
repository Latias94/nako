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

## M3: Metadata and NFO

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

## M4: Playback MVP

Outcome: Taru can serve local media through direct play and a minimal HLS
transcode path.

Deliverables:

- Playback decision model.
- Direct play route.
- FFmpeg transcode session manager.
- Minimal HLS segment route.
- Transcode temp directory and cleanup policy.

Exit criteria:

- Compatible files direct play without FFmpeg.
- An incompatible fixture produces playable HLS output.
- Session cancellation cleans up process and temporary files.

## M5: Extension Surface

Outcome: Taru has stable external automation and addon surfaces.

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
