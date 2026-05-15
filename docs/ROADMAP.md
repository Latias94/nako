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

### Playback and Transcode: M4.2-M4.10

Status: completed for the local video-library playback MVP.

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
- HTTP remux playback route backed by the remux application service.
- persisted remux/transcode session records with startup stale-session
  recovery and lookup API.
- minimal single-variant HLS command planning, session orchestration, playlist
  route, and segment route.
- hardware acceleration capability, fallback, and CPU/GPU budget policy for
  VAAPI, NVENC, and QuickSync command planning.
- MVP stabilization for API docs, config docs, route error behavior, test
  coverage, known limitations, and bounded resource notes.

Recommended next goal:

- M5 extension and automation surface.

Future playback work:

- remote-source staging/cache behavior for FFmpeg.
- adaptive bitrate HLS ladder.

### Extension and Automation: M5

Status: active, completed through M5.1 event outbox foundation.

This phase turns the early architectural decisions into a usable external
surface:

- webhook outbox and delivery policy;
- automation job model for API-key backed providers;
- Taru addon manifest schema;
- addon resource routes and response envelopes;
- timeout, retry, authentication, and trust model;
- one reference addon.

Completed:

- M5.0 design baseline with ADRs for durable event outbox, webhook/automation
  trigger policy, capability-scoped HTTP addons, external provider boundaries,
  resource classes, and security constraints.
- M5.1 event outbox foundation with domain event records, SQLite persistence,
  repository boundary, idempotent enqueue behavior, and write points from scan,
  metadata, NFO, and playback session completion.

Recommended next goal:

- M5.2 webhook delivery worker.

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

`server-foundation` was the initial planning hub. M5 has split
`addons-automation` into its own active workstream. As implementation grows,
split the remaining broad domains into narrower workstreams:

- `playback-streaming`: direct play, remux, HLS, transcode, hardware policy.
- `metadata-catalog`: providers, NFO, catalog graph, artwork, search.
- `storage-vfs`: local/remote backends, directory cache, byte-range cache.
- `clients`: future Flutter and web client contracts.

Do future splits when a domain needs independent milestones or ADRs. Do not
split merely because a domain exists conceptually.
