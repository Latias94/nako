# Taru Roadmap

Taru is currently a Rust modular monolith focused on the self-hosted media
server backend. The roadmap is intentionally staged so storage, metadata,
playback, search, automation, and future clients can grow without collapsing
into a single tightly coupled crate.

## Phase Bands

### Foundation: M0-M2.1

Status: completed.

The repository has a Rust workspace, crate boundaries, SQLite persistence,
minimal server runtime, persisted jobs, pagination, logging, local setup,
testing strategy, and license notes.

### Metadata and Catalog: M3.1-M4.1

Status: completed for the first movie-focused slice.

Taru can scan local sources, persist metadata, merge TMDB/NFO inputs through
local-authority rules, hydrate a normalized catalog graph, rebuild search
projection, and expose browse APIs for items, people, tags, genres, credits,
and images.

Important remaining breadth:

- TMDB series, season, and episode support.
- Douban provider MVP.
- Bangumi provider MVP.
- item-level metadata profile overrides.
- image proxy/cache routes and preview-frame generation jobs.

### Playback and Transcode: M4.2-M4.x

Status: active.

Completed:

- playback decision model;
- direct play byte-range route and HEAD preflight;
- direct play planning boundary;
- FFmpeg copy-remux command planning;
- remux session lifecycle model;
- remux FFmpeg process runner with cancellation, timeout, concurrency guard,
  temporary output cleanup, and server runtime budget configuration.
- remux application service with local staging, deterministic output naming,
  completed-output reuse, in-flight duplicate conflict behavior, and API-safe
  error mapping.

Recommended next goal:

- M4.6: remux playback route.

Future playback work:

- HTTP route for remuxed playback;
- persisted transcode/remux session records;
- HLS transcode session manager;
- hardware acceleration detection and encode policy for VAAPI, NVENC, and QSV;
- CPU/GPU queue policy;
- remote-source staging/cache behavior for FFmpeg.

### Extension and Automation: M5

Status: planned.

This phase turns the early architectural decisions into a usable external
surface:

- webhook outbox and delivery policy;
- automation job model for API-key backed providers;
- Taru addon manifest schema;
- addon resource routes and response envelopes;
- timeout, retry, authentication, and trust model;
- one reference addon.

### Remote Storage and VFS Expansion: M6

Status: planned.

This phase proves that remote sources are first-class storage backends instead
of pretending to be local paths:

- WebDAV or S3-compatible backend preview;
- directory metadata cache;
- byte-range cache or local staging path;
- remote listing rate limits and retry policy;
- remote-source playback policy.

### Client and Product Experience: M7+

Status: intentionally deferred.

The likely first client target is Flutter, but the server should expose stable
API contracts and predictable media URLs before client work dominates. Client
planning should start after the browse and playback surfaces are coherent.

## Workstream Split Direction

`server-foundation` is still the active workstream, but it is carrying several
large domains. As implementation grows, split it into narrower workstreams:

- `playback-streaming`: direct play, remux, HLS, transcode, hardware policy.
- `metadata-catalog`: providers, NFO, catalog graph, artwork, search.
- `storage-vfs`: local/remote backends, directory cache, byte-range cache.
- `addons-automation`: webhooks, external automation, addon protocol.
- `clients`: future Flutter and web client contracts.

Do the split when a domain needs independent milestones or ADRs. Do not split
just to make the directory tree look complete.
