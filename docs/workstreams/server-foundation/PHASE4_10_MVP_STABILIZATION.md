# Phase 4.10: MVP Stabilization

Status: completed.

## Goal

Stabilize the local video-library playback MVP before expanding into extension,
automation, or remote-storage work.

## Completed Shape

- Audited HTTP API documentation against implemented routes and response
  shapes.
- Audited error mapping for playback, session, scan, metadata, and NFO routes.
- Documented runtime configuration for scan, metadata, remux, HLS, hardware
  policy, and resource budgets.
- Added focused tests for uncovered HLS session readiness edge cases.
- Documented known MVP limitations and intentional non-goals.
- Checked performance-sensitive paths for bounded concurrency and to avoid
  unbounded file or artwork loading.

## Non-Goals

- No new provider implementation.
- No Flutter/client work.
- No remote storage backend implementation.
- No addon runtime implementation.

## Known MVP Limitations

- Playback, probe, remux, and HLS execution currently require local source
  paths. Remote source staging/cache behavior is reserved for M6.
- HLS is a minimal single-variant path. Adaptive bitrate ladders, subtitle
  packaging, and multi-audio renditions are future playback work.
- Hardware acceleration is modeled through capability, policy, fallback,
  command planning, and resource budgets. Real GPU smoke tests are outside the
  deterministic workspace test suite.
- Search uses the current SQLite projection fallback. FTS ranking/tokenization
  and optional Tantivy/Meilisearch adapters are future search work.
- Image routes expose metadata records only. Image proxy/cache routes and
  preview-frame generation jobs are future catalog/artwork work.
- Metadata provider breadth is intentionally narrow: TMDB movie support and NFO
  local metadata workflows exist, while TMDB series, Douban, and Bangumi are
  future goals.
- Client ergonomics are intentionally deferred. The server API is the stable
  surface for future Flutter or web clients.
- Addons, webhook delivery, and external automation are planned for M5.

## Validation

Coverage:

- docs match implemented API routes and config fields;
- validation gates pass for the workspace;
- route errors are stable and safe for clients;
- MVP limitations are explicitly documented;
- no known unbounded expensive playback or scan path remains undocumented.
