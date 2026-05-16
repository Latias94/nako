# Taru Roadmap

Taru is currently a Rust modular monolith focused on the self-hosted media
server backend. The roadmap is intentionally staged so storage, metadata,
playback, search, automation, and future clients can grow without collapsing
into a single tightly coupled crate.

Goal numbers are historical identifiers. Earlier gaps such as M10-M12 and M17
are not reused; new work uses the next number after the highest documented
milestone.

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

Later follow-up:

- M13-M23 completed the metadata, runtime, database, storage, ingestion, and
  API boundary hardening needed before the M24 server architecture pass.

### Operational and Boundary Hardening: M13-M23

Status: completed.

After the first remote playback and multi-library hardening waves, Taru added
several focused operational cleanup phases before the final server composition
pass:

- M13-M14 metadata maintenance, scheduling, provider diagnostics, and raw-cache
  lifecycle.
- M15-M16 runtime foundation, SQLite/migration behavior, secret redaction,
  hardware selection policy, storage backend registry, and staged-input lease
  lifecycle.
- M18 metadata provider runtime productization.
- M19 database boundary hardening.
- M20 server test-surface decomposition.
- M21 storage backend registry documentation in the server-foundation stream.
- M22 ingestion failure diagnostics.
- M23 API, HTTP router, and DB boundary cleanup.

The detailed evidence lives in the `metadata-operations`, `runtime-foundation`,
and `server-foundation` workstreams.

### Server Architecture Hardening: M24

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

- M24.0 design baseline with ADR 0019, a dedicated
  `server-architecture-hardening` workstream, M24 milestone split, and audit
  notes for server composition, runtime ownership, repository boundaries, NFO,
  catalog hydration, and obsolete helpers.
- M24.1-M24.4 implementation has decomposed `TaruApp` into focused service
  handles, added runtime supervisor ownership for detached workers, moved
  catalog graph hydration behind an atomic repository operation, removed
  temporary root-app forwards, and replaced hand-written NFO XML parsing with a
  `roxmltree` parser boundary.

Completed:

- M25 transcode runtime productization.

### Transcode Runtime Productization: M25

Status: completed.

This phase turns the existing remux/HLS MVP into a cleaner playback and
transcode runtime boundary before client work depends on it:

- split playback application code into focused direct-play, remux, HLS,
  staging, and runtime modules;
- replace the CPU-only detector with FFmpeg-backed hardware capability probing
  when hardware acceleration is configured;
- make VAAPI, NVENC, and QuickSync selection, fallback, and resource budgets
  explicit contracts;
- define stable playback session lifecycle, error categories, and client-facing
  URL behavior;
- keep adaptive bitrate HLS ladder as a follow-up after the runtime boundary is
  clean.

Completed:

- M25.1 split playback orchestration into focused direct-play, FFmpeg input,
  remux, and HLS app modules.
- M25.2 replaced CPU-only runtime detection with FFmpeg-backed encoder
  capability probing for VAAPI, NVENC, and QuickSync/QSV.
- M25.3 documented the session lifecycle and validated acceleration fallback,
  fail-fast policy, CPU/GPU resource budgets, runner timeout/cancellation, and
  stale-startup recovery behavior.

### Playback Client Contract: M26

Status: completed.

This phase hardens the server contract that future web or Flutter clients will
depend on before client UI work dominates:

- keep playback session inspection on a stable `TranscodeSessionResponse`
  envelope;
- add a public playback session cancellation route backed by live remux/HLS
  runner cancellation tokens;
- document active and terminal playback session states, cancellation conflict
  behavior, and error DTOs;
- validate the route behavior with focused HTTP tests.

Future client and product experience work remains intentionally deferred until
the server browse and playback contracts are coherent.

### Metadata-Catalog Expansion: M27

Status: completed for M27.0 design baseline, M27.1 schema/repository slice,
M27.2 local inference/provisional hierarchy slice, and M27.3 provider/NFO
expansion.

M27.0 turned the movie-first metadata/catalog foundation into a video-first
media-server model using the project language in `CONTEXT.md` and ADR 0021:

- treat **Media Library**, **Media Domain**, **Library Preset**, **Media
  Source**, **Media Item**, **Canonical Metadata**, **Media Technical Facts**,
  **Metadata Source Priority**, **NFO Round Trip**, **Managed Artwork**, and
  **Source Duplicate Relationship** as first-class planning terms;
- split the remaining metadata, NFO, artwork, and search follow-ups out of the
  historical `server-foundation` backlog into a dedicated
  `metadata-catalog` workstream;
- design the movie, series, season, episode, **Episode-Like Item**,
  **Extra Item**, and **Franchise Collection** model before adding TMDB series,
  Douban, or Bangumi breadth;
- map TMDB, Douban, Bangumi, and future provider concepts through **Provider
  Subject** and **Provider Mapping**, not provider-owned item identity;
- define stable **Browse Facets** and **Sort Keys** instead of exposing raw
  database columns as client query contracts;
- preserve local/NFO authority and library-scoped policy while opening a clean
  path for provider and addon metadata contributions.

Completed implementation slice:

- M27.1 catalog schema and repository slice persisted **Provider Subject**,
  **Provider Mapping**, **Source Duplicate Relationship**, and **Local
  Inference Evidence** records through `taru-core` records, `taru-db`
  migration `0018_metadata_catalog_domain.sql`, repository traits, SQLite
  adapters, and focused repository tests.
- M27.2 local inference and provisional hierarchy connected scan indexing to
  source-owned **Local Inference Evidence**, added weak-name **Unknown Media
  Item** fallback, keeps inference evidence as a current source/version
  snapshot, preserves confirmed canonical metadata during rescan, and creates
  provisional series/season/episode hierarchy without provider, NFO, Source
  Variant UI, or browse API breadth.
- M27.3 hierarchy confirmation and provider/NFO expansion added a shared
  confirmation service, in-place provisional hierarchy confirmation, accepted
  provider mapping writes for TMDB/Douban/Bangumi metadata refreshes, TMDB
  series/season/episode fetch support, and NFO episode hierarchy confirmation.

Recommended next goal:

- M28 crate boundary and public protocol hardening.

### Crate Boundary And Public Protocol Hardening: M28

Status: in progress.

This phase deepens Taru's crate and module seams so public client wire types
can live in a permissive protocol crate while server internals remain AGPL and
the large workflow crates become easier to navigate.

Planned slices:

- M28.0 boundary baseline and scope freeze.
- M28.1 public client protocol extraction.
- M28.2 core module deepening and repository seam narrowing.
- M28.3 library and NFO module decomposition.
- M28.4 playback seam clarification.
- M28.5 closeout and follow-on split.

Recommended next goal:

- M28.0 boundary baseline and scope freeze.

## Workstream Split Direction

`server-foundation` was the initial planning hub. M5 split
`addons-automation` into its own completed workstream, M6 split `storage-vfs`,
M7 split `playback-streaming`, M13-M19 used metadata and runtime operations for
specialized hardening, M24 split `server-architecture-hardening` for the server
composition pass, M25 split `transcode-runtime` for playback runtime
productization, and M27 split `metadata-catalog` for media-library domain
expansion. As implementation grows, split the remaining broad domains
into narrower workstreams:

- `clients`: future Flutter and web client contracts.

Do future splits when a domain needs independent milestones or ADRs. Do not
split merely because a domain exists conceptually.
