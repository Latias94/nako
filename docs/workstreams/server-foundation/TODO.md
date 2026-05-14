# Server Foundation TODO

## Architecture

- [x] Define workspace crate layout and dependency direction.
- [x] Create initial `taru-core` domain model draft.
- [x] Decide default database target for MVP.
- [x] Define service trait boundaries for library, metadata, VFS, transcode,
      search, events, automation, and addons.
- [x] Add ADR index and first accepted/rejected decisions.
- [x] Define shared job cancellation and retry policy.
- [x] Define shared job state and resource-budget model.
- [x] Add observability conventions for async pipelines and background jobs.
- [x] Add persisted job lifecycle state for server runtime work.
- [x] Add persisted job input payloads for audit and future retry.
- [x] Add minimal HTTP API for health, libraries, sources, items, probes, and jobs.
- [x] Add pagination foundation for list endpoints.
- [x] Add structured logging initialization for the server binary.

## VFS and Storage

- [x] Draft `StorageBackend` and `VirtualFile` traits.
- [x] Define storage capability flags.
- [x] Define stable media locator format.
- [ ] Design directory metadata cache.
- [ ] Design byte-range media cache.
- [ ] Document local filesystem backend behavior.
- [ ] Document WebDAV/S3/rclone integration paths.
- [ ] Decide how hard links and soft links are represented for non-local
      storage backends.

## Metadata and NFO

- [x] Define canonical metadata fields.
- [x] Define external ID model for TMDB, Douban, Bangumi, IMDb, and local IDs.
- [x] Define provider priority and field-level lock policy.
- [x] Define raw provider response cache policy.
- [x] Define NFO import/export compatibility targets.

## Streaming and Transcoding

- [x] Define media probe output model.
- [x] Persist media probe output and stream details.
- [x] Define bounded probe pipeline and default concurrency.
- [ ] Define playback decision model.
- [ ] Define direct play, remux, and transcode decision rules.
- [ ] Draft FFmpeg command builder interface.
- [ ] Define hardware acceleration detection model for VAAPI, NVENC, and QSV.
- [ ] Define CPU/GPU transcode concurrency and queue policy.
- [ ] Define remote source staging/cache behavior for FFmpeg.

## Addons and Automation

- [ ] Draft Taru addon manifest schema.
- [ ] Define addon resource routes and response envelopes.
- [ ] Define addon timeout, retry, authentication, and trust model.
- [ ] Define automation job model.
- [ ] Define API-key provider secret storage requirements.
- [ ] Define webhook event envelope and delivery policy.

## Documentation

- [x] Create MVP milestone document.
- [x] Create initial API design notes.
- [x] Create local development setup guide.
- [x] Create test strategy for crate-level and integration tests.
- [x] Create licensing notes for reference-only GPL code.
