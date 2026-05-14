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
