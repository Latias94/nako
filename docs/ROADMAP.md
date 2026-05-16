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

Next phase:

- M5 extension and automation surface. Completed.

Future playback work:

- adaptive bitrate HLS ladder.

### Extension and Automation: M5

Status: completed.

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
- M5.2 webhook delivery worker with endpoint configuration, delivery attempts,
  signed webhook envelopes, retry/backoff state, `webhook_concurrency`
  budgeting, explicit event dispatch, and delivery inspection APIs.
- M5.3 automation job model with external provider configuration, automation
  job inputs/summaries, generated artifact persistence, timeout/cancellation
  runner boundary, and provider/job/artifact inspection APIs.
- M5.4 addon manifest and resource contract with protocol versioning, resource
  envelopes, scope grants, disabled-by-default registration, bounded addon
  HTTP caller, SQLite persistence, and registration/inspection APIs.
- M5.5 reference addon and stabilization with a local addon fixture, end-to-end
  register/query/call test, addon/webhook/automation guides, and documented M5
  limitations.

Next phase:

- M6 remote storage and VFS expansion. Completed.

### Remote Storage and VFS Expansion: M6

Status: completed.

This phase proves that remote sources are first-class storage backends instead
of pretending to be local paths:

- WebDAV read-only backend preview;
- directory and stat cache;
- remote byte-range reads and local staging path;
- remote listing rate limits and retry policy;
- remote-source playback policy.

Completed:

- M6.0 design baseline with ADR 0016, a dedicated `storage-vfs` workstream,
  local-path dependency audit, WebDAV-first decision, and M6 milestone split.
- M6.1 WebDAV read-only VFS backend.
- M6.2 directory/stat cache and stale-cache tombstone protection.
- M6.3 remote probe staging.
- M6.4 remote playback policy.
- M6.5 remote storage stabilization.

Recommended next goal:

- M7 playback streaming and remote hardening. Completed.

### Playback Streaming and Remote Hardening: M7

Status: completed.

This phase hardens the remote playback path after the M6 WebDAV preview:

- remote direct response-body streaming;
- staging manifest, disk budget, and cleanup;
- precise playback/storage error mapping;
- remote playback stream and staging resource budgets;
- multi-library and multi-remote backend configuration.

Completed:

- M7.0 design baseline with ADR 0017, a dedicated `playback-streaming`
  workstream, M7 milestone split, and ownership for M6 deferred playback
  hardening tasks.
- M7.1 foundation for remote direct response-body streaming, including VFS
  `ReadStream`, WebDAV byte streams, direct-play stream bodies, HEAD preflight,
  and initial playback app/http module split.
- M7.2 foundation for durable staging manifest persistence, including core
  records, repository contract, SQLite migration, a dedicated DB module,
  remote probe input manifest recording, and remux/HLS FFmpeg input manifest
  recording.
- M7.2 disk budget wiring with `[staging].max_bytes` and app-side preflight
  checks before remote probe or FFmpeg input staging.
- M7.2 startup cleanup for expired staged inputs, with active-lease protection.
- M7.3 first HTTP error-mapping slice for staging budget, staging validation,
  storage timeout/auth/rate-limit, and FFmpeg provider failures.
- M7.4 resource-budget foundation with independent remote stream and stage
  concurrency limits.
- NFO import/export now use the configured VFS backend boundary and gate export
  on writable storage capabilities.
- M7.5 multi-library backend foundation with `[[libraries]]` as the only
  library configuration shape, startup persistence for all configured
  libraries, and `MediaSource.library_id` for library-aware backend resolution.
- M7.6 stabilization audit mapping the full M7 objective to concrete evidence,
  known limitations, and validation gates.

Recommended next goal:

- M8 multi-library correctness and operational hardening. Completed.

### Multi-Library Correctness and Operational Hardening: M8

Status: completed.

This phase made multi-library operation data-safe before the next server
architecture pass:

- source locator identity is scoped by library;
- source lookup by locator requires library identity;
- CLI scan/list commands expose explicit library selection;
- staging budget reservation is serialized across check, stage, and manifest
  recording;
- panic-style default library helpers were replaced with explicit config
  selection.

Completed:

- [Phase 8.0](workstreams/multi-library-hardening/PHASE8_0_CORRECTNESS_BASELINE.md)
  records source identity, CLI, and staging budget invariants.

Recommended next goal:

- M9 server architecture hardening before expanding metadata providers,
  clients, or plugin/runtime surfaces.

### Server Architecture Hardening: M9

Status: completed.

This phase turns the server into a cleaner modular-monolith composition
boundary:

- `taru-server::app::TaruApp` becomes a thin composition root;
- workflow orchestration moves into focused application services;
- background jobs and cleanup loops register through an explicit runtime
  supervisor or worker registry;
- high-level services use narrow ports or service handles instead of broad
  concrete store access;
- multi-record writes get explicit repository or unit-of-work boundaries;
- obsolete MVP helpers and compatibility paths are deleted once replacements
  are covered.

Completed:

- M9.0 design baseline with ADR 0019, a dedicated
  `server-architecture-hardening` workstream, M9 milestone split, and audit
  notes for server composition, runtime ownership, repository boundaries, NFO,
  catalog hydration, and obsolete helpers.
- M9.1-M9.4 implementation has decomposed `TaruApp` into focused service
  handles, added runtime supervisor ownership for detached workers, moved
  catalog graph hydration behind an atomic repository operation, removed
  temporary root-app forwards, and replaced hand-written NFO XML parsing with a
  `roxmltree` parser boundary.

Recommended next goal:

- after M9 stabilization, continue provider/runtime productization and client
  contract planning on top of the cleaned server boundary.

### Client and Product Experience: M10+

Status: intentionally deferred.

The likely first client target is Flutter, but the server should expose stable
API contracts and predictable media URLs before client work dominates. Client
planning should start after the browse and playback surfaces are coherent.

## Workstream Split Direction

`server-foundation` was the initial planning hub. M5 split
`addons-automation` into its own completed workstream, M6 split `storage-vfs`,
M7 split `playback-streaming`, and M9 split
`server-architecture-hardening` for the active server composition and fearless
refactor pass. Existing runtime and metadata operations workstreams continue to
track later specialized hardening. As implementation grows, split the
remaining broad domains into narrower workstreams:

- `metadata-catalog`: providers, NFO, catalog graph, artwork, search.
- `clients`: future Flutter and web client contracts.

Do future splits when a domain needs independent milestones or ADRs. Do not
split merely because a domain exists conceptually.
