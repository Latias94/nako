# Server Foundation TODO

## Architecture

- [ ] Define workspace crate layout and dependency direction.
- [ ] Create initial `taru-core` domain model draft.
- [ ] Decide default database target for MVP.
- [ ] Define service trait boundaries for library, metadata, VFS, transcode,
      search, events, automation, and addons.
- [ ] Add ADR index and first accepted/rejected decisions.
- [ ] Define shared job state, cancellation, retry, and resource-budget model.
- [ ] Add observability conventions for async pipelines and background jobs.

## VFS and Storage

- [ ] Draft `StorageBackend` and `VirtualFile` traits.
- [ ] Define storage capability flags.
- [ ] Define stable media locator format.
- [ ] Design directory metadata cache.
- [ ] Design byte-range media cache.
- [ ] Document local filesystem backend behavior.
- [ ] Document WebDAV/S3/rclone integration paths.
- [ ] Decide how hard links and soft links are represented for non-local
      storage backends.

## Metadata and NFO

- [ ] Define canonical metadata fields.
- [ ] Define external ID model for TMDB, Douban, Bangumi, IMDb, and local IDs.
- [ ] Define provider priority and field-level lock policy.
- [ ] Define raw provider response cache policy.
- [ ] Define NFO import/export compatibility targets.

## Streaming and Transcoding

- [ ] Define media probe output model.
- [ ] Persist media probe output and stream details.
- [ ] Define bounded probe pipeline and default concurrency.
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

- [ ] Create MVP milestone document.
- [ ] Create initial API design notes.
- [ ] Create local development setup guide.
- [ ] Create test strategy for crate-level and integration tests.
- [ ] Create licensing notes for reference-only GPL code.
