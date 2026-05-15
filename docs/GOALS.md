# Taru Goal Map

This file is the top-level tracker for current and upcoming engineering goals.
Workstream TODO files track tasks; this file tracks goal boundaries,
non-goals, exit criteria, and evidence.

## Goal Format

Each implementation goal should define:

- Objective: the user-visible or architecture-visible outcome.
- Deliverables: concrete files, APIs, crates, or documents expected to change.
- Non-goals: adjacent work intentionally left out.
- Exit criteria: observable behavior that must be true.
- Evidence: commands, tests, docs, or commit IDs proving completion.

Use one goal per meaningful milestone. A goal should be large enough to produce
a coherent commit, but small enough that validation remains clear.

## Completed Goals

### M0-M2.1: Server Runtime Foundation

Status: completed.

Evidence:

- Rust workspace and crate stubs exist.
- SQLite persistence, server runtime, persisted jobs, pagination, logging, and
  developer docs are implemented.
- Related docs: [server-foundation milestones](workstreams/server-foundation/MILESTONES.md).

### M3.1-M3.6: Metadata, NFO, Profiles, and Catalog Planning

Status: completed for the first movie-focused foundation.

Evidence:

- Metadata merge policy, NFO policy, provider secret policy, library presets,
  catalog graph, scan state, and artwork resource-class ADRs exist.
- TMDB movie refresh, NFO import/export jobs, metadata profile execution, and
  catalog/search planning are implemented or documented.

### M4.0-M4.10: Catalog Ingestion and Playback MVP

Status: completed.

Evidence:

- Catalog ingestion, graph hydration, browse APIs, direct play, FFmpeg command
  planning, remux process runner guard, and remux application service
  integration are implemented.
- HTTP remux playback route is implemented.
- Remux/transcode session records are persisted in SQLite and exposed through
  an app/API lookup path.
- A minimal single-variant HLS transcode path can generate, persist, and serve
  playlists and segments.
- Hardware acceleration capability, policy, fallback, and resource-budget
  models are implemented without requiring real GPU hardware in tests.
- MVP stabilization audited API docs, config docs, error behavior, test gaps,
  performance constraints, and known limitations.
- Last completed implementation goal: M4.10 MVP stabilization.

## Recently Completed Goals

### Planning Docs: Goal Map and Refactoring Policy

Status: completed.

Objective:

- Give the project a single top-level route for roadmap, goal tracking,
  workstream ownership, and fearless refactoring policy.

Deliverables:

- `docs/README.md`
- `docs/ROADMAP.md`
- `docs/GOALS.md`
- `docs/workstreams/README.md`
- `docs/development/REFACTORING_POLICY.md`
- server-foundation milestone and TODO updates

Non-goals:

- no runtime code changes;
- no ADR status migration beyond documenting the hygiene rule;
- no workstream directory split yet.

Exit criteria:

- top-level docs link to current focus, roadmap, and active workstream;
- the next recommended implementation goal is explicit;
- refactoring policy documents crate boundaries, dependency direction, and
  validation gates;
- doc consistency checks pass.

Evidence:

- `git diff --check` passed for the docs-only change set.

### M4.5: Remux App Service Integration and Local Staging Policy

Status: completed.

Evidence:

- `taru-server::app` has a remux application service boundary.
- `remux_staging_root` config defines the local staging root.
- Remux outputs are deterministic by source ID and container.
- Completed staged outputs are reused.
- In-flight duplicate requests return `Conflict`.
- Tests cover app-service runner execution, completed-output reuse, duplicate
  conflict behavior, staging path validation, and config defaults.

### M4.6: Remux Playback Route

Status: completed.

Evidence:

- `GET /sources/{source_id}/stream/remux` is implemented.
- The handler calls the remux app service and streams staged output.
- `output_container=mp4|mkv` selects the staged remux container.
- Completed staged outputs are reused.
- In-flight duplicates map to `409 conflict`.
- Tests cover range streaming, completed-output reuse, duplicate conflict, and
  unchanged direct play behavior.

### M4.7: Playback Session Persistence

Status: completed.

Evidence:

- `transcode_sessions` persists remux and future transcode session state.
- Remux app-service requests create planned sessions, mark running sessions,
  and persist finished, failed, cancelled, and stale recovery states.
- Completed persisted remux sessions are reused after app restart.
- Active persisted sessions drive duplicate `409 conflict` behavior.
- `GET /playback/sessions/{session_id}` exposes current persisted state.

### M4.8: HLS Transcode Foundation

Status: completed.

Evidence:

- `taru-transcode` plans and runs minimal single-variant HLS sessions through
  FFmpeg.
- HLS output uses a staging layout with temporary directory promotion.
- HLS app service uses persisted transcode sessions for planned, running,
  finished, failed, cancelled, stale, duplicate, and reuse behavior.
- `GET /sources/{source_id}/stream/hls/playlist.m3u8` returns a rewritten HLS
  playlist.
- `GET /playback/sessions/{session_id}/hls/segments/{segment_name}` serves
  generated segments with path traversal protection.

### M4.9: Hardware Acceleration Policy

Status: completed.

Evidence:

- `taru-transcode` has a hardware acceleration capability report, detector
  boundary, policy selection, fallback behavior, and resource-budget model.
- HLS command planning can select CPU-only, VAAPI, NVENC, or QuickSync encoder
  arguments without requiring real hardware in tests.
- `taru-server` config exposes hardware acceleration, fallback, CPU slots, and
  GPU slots with conservative defaults.
- HLS app-service concurrency uses CPU/GPU resource budgets based on the
  selected acceleration class.

### M4.10: MVP Stabilization

Status: completed.

Evidence:

- HTTP API docs match the current local playback routes, including remux, HLS,
  persisted session lookup, and playback error behavior.
- Local setup docs cover scan, probe, metadata, remux, HLS staging, hardware
  policy, and CPU/GPU resource budget configuration.
- Test strategy docs reflect current coverage for browse, metadata/NFO, direct
  play, remux, HLS, persisted playback sessions, and hardware policy.
- Known MVP limitations are documented in the phase note.
- Focused HLS session readiness tests cover active-session conflict behavior at
  the app and HTTP layers.

## Recently Completed Goal

### M5: Extension and Automation Surface

Status: completed.

Implement webhook outbox, automation jobs, addon manifest schema, and one
reference addon. Keep AI-like experience improvements as explicit external
provider/API-key workflows rather than local model or vector infrastructure.

Deliverables:

- M5.0 Extension/Automation Design Baseline.
- M5.1 Event Outbox Foundation.
- M5.2 Webhook Delivery Worker.
- M5.3 Automation Job Model.
- M5.4 Addon Manifest and Resource Contract.
- M5.5 Reference Addon and Stabilization.

Non-goals:

- no local model runtime or vector database;
- no in-process native plugin ABI;
- no embedded JavaScript runtime in the first M5 slice;
- no remote storage backend implementation.

Evidence for M5.0:

- [ADR 0014](adr/0014-durable-event-outbox-for-webhooks-and-automation.md)
  documents durable event outbox and webhook/automation trigger policy.
- [ADR 0015](adr/0015-capability-scoped-http-addons-and-automation-providers.md)
  documents capability-scoped HTTP addons and external automation providers.
- [addons-automation workstream](workstreams/addons-automation/README.md)
  tracks M5 milestones, TODOs, phase notes, resource classes, and security
  boundaries.

Evidence for M5.1:

- `taru-core` defines domain event kinds, event subjects, outbox status, event
  records, and `EventOutboxRepository`.
- `taru-db` migration `0009_event_outbox.sql` persists durable outbox events
  with idempotency by event kind and key.
- `taru-server` writes outbox events for successful library scan, metadata
  refresh, NFO import/export, and playback session completion paths.
- Tests cover outbox persistence, idempotency, and payload safety constraints
  against plaintext secrets and raw local paths.

Evidence for M5.2:

- `taru-core` defines webhook endpoint configuration, delivery attempt records,
  statuses, and `WebhookRepository`.
- `taru-db` migration `0010_webhooks.sql` persists webhook endpoints and
  delivery attempts with per-event inspection.
- `taru-events` builds versioned webhook envelopes, signs payloads with
  HMAC-SHA256, enforces request timeouts, records failed attempts with retry
  timestamps, and provides a `reqwest` transport.
- `taru-server` exposes webhook endpoint configuration/inspection, per-event
  delivery-attempt inspection, explicit outbox event dispatch, and
  `webhook_concurrency` resource budgeting.
- Tests cover SQLite persistence, signed success delivery, failed retry state,
  real transport delivery to a mocked local webhook server, and HTTP
  configuration/inspection routes.

Evidence for M5.3:

- `taru-core` defines automation provider configuration, automation
  capabilities, job input/summary envelopes, artifact records, and
  `AutomationRepository`.
- `taru-db` migration `0011_automation.sql` persists provider configuration and
  generated artifacts.
- `taru-automation` runs mockable external providers through a timeout and
  cancellation-aware runner, persists proposed artifacts, writes job summaries,
  and rejects implicit canonical metadata mutation.
- `taru-server` exposes provider configuration, automation job enqueue, and
  artifact inspection APIs without calling external providers inline.
- Tests cover provider/artifact persistence, mocked provider execution, secret
  omission from job input, canonical-mutation rejection, and HTTP enqueue and
  inspection routes.

Evidence for M5.4:

- `taru-addon-protocol` defines the manifest, protocol version, resource
  declarations, scopes, auth modes, request/response envelopes, mockable
  transport, `ReqwestAddonTransport`, and bounded resource caller.
- `taru-core` defines addon registration status and records plus
  `AddonRepository`.
- `taru-db` migration `0012_addons.sql` persists addon registrations, manifest
  snapshots, granted scopes, and enabled/disabled status.
- `taru-server` exposes addon registration, list, status-filtered list, and
  detail APIs. Registrations are disabled by default and rejected when the
  manifest or granted scopes do not satisfy the resource contract.
- Tests cover manifest validation, invalid manifest rejection, scope denial,
  auth token enforcement, bounded retry behavior, response envelope mapping,
  persistence, and HTTP registration/inspection routes.

Evidence for M5.5:

- `taru-reference-addon` provides a minimal local metadata addon fixture with
  a valid manifest and HTTP resource route.
- `taru-server` end-to-end tests register the reference addon through
  `POST /addons`, query it through `GET /addons/{addon_id}`, and call the
  metadata resource through `ReqwestAddonTransport`.
- Addon author, webhook receiver, and automation provider guides document the
  current extension surface.
- [Phase 5.5](workstreams/addons-automation/PHASE5_5_REFERENCE_ADDON_STABILIZATION.md)
  documents M5 known limitations and stabilization evidence.

### M6.0: Remote Storage and VFS Design Baseline

Status: completed.

Objective:

- Define the remote-storage architecture before adding WebDAV or S3-compatible
  backend code.

Deliverables:

- ADR 0016 for remote storage and VFS cache boundaries.
- Dedicated `storage-vfs` workstream.
- Local-path dependency audit for `taru-vfs`, scan/probe, direct play, remux,
  and HLS.
- M6 milestone split with WebDAV selected as the first backend preview.
- Roadmap, goal map, ADR index, and workstream index updates.

Evidence:

- [ADR 0016](adr/0016-remote-storage-and-vfs-cache-boundary.md) documents
  WebDAV-first remote storage, VFS cache, staging, credential, and local-path
  boundaries.
- [storage-vfs workstream](workstreams/storage-vfs/README.md) owns M6 remote
  storage, VFS cache, remote staging, and playback policy work.
- [Phase 6.0](workstreams/storage-vfs/PHASE6_0_REMOTE_STORAGE_DESIGN_BASELINE.md)
  records the local-path dependency audit and M6 milestone split.

### M6.1: WebDAV Read-Only VFS Backend

Status: completed.

Evidence:

- `taru-vfs::WebDavBackend` implements read-only `stat`, `list`, and
  `open_range`.
- `VfsLibraryScanner` can scan a mocked WebDAV library without plaintext
  credentials in source locators.
- [Phase 6.1](workstreams/storage-vfs/PHASE6_1_WEBDAV_READ_ONLY_BACKEND.md)
  records validation and limitations.

### M6.2: Directory and Stat Cache

Status: completed.

Evidence:

- `taru-core` defines VFS cache object, listing, failure, and repository
  contracts.
- `taru-db` migration `0013_vfs_cache.sql` persists cached stat/list metadata
  and transient failure state.
- `taru-vfs::CachedStorageBackend` reuses fresh cache and serves stale cache on
  transient storage errors.
- `LibraryIndexService` skips tombstoning when a scan used stale VFS cache.
- [Phase 6.2](workstreams/storage-vfs/PHASE6_2_DIRECTORY_STAT_CACHE.md)
  records validation and remaining cache gaps.

### M6.3: Remote Probe Staging

Status: completed.

Evidence:

- `taru-vfs` defines `StageRequest`, `StagedFile`, deterministic staging paths,
  and `StorageBackend::stage`.
- `taru-vfs::WebDavBackend` can stage a remote media object to a deterministic
  local path and reuse it when size still matches.
- `LibraryProbeService` uses staging when a backend returns no local path hint.
- [Phase 6.3](workstreams/storage-vfs/PHASE6_3_REMOTE_PROBE_STAGING.md)
  records validation and remaining staging gaps.

### M6.4: Remote Playback Policy

Status: completed.

Evidence:

- `StorageBackend::read_range` gives direct play a VFS byte path when a source
  has no local path hint.
- `taru-vfs::WebDavBackend` uses HTTP `Range` GET for byte windows.
- Remux and HLS input planning stages remote sources under
  `remux_staging_root/inputs` before invoking FFmpeg.
- Tests cover remote direct-play bytes, remote FFmpeg staging, local path-hint
  reuse, WebDAV range GET, and WebDAV staging.
- [Phase 6.4](workstreams/storage-vfs/PHASE6_4_REMOTE_PLAYBACK_POLICY.md)
  records validation and remaining production config/API gaps.

### M6.5: Remote Storage Stabilization

Status: completed.

Evidence:

- `TaruServerConfig` supports `[library.webdav]` preview configuration with
  WebDAV root, base URL, username, password environment reference, timeout,
  and retry attempt limits.
- `taru-server::app` builds configured WebDAV storage through
  `WebDavBackend` wrapped in `CachedStorageBackend`.
- Configured WebDAV library scan/probe uses the WebDAV root from
  `library_from_config`; remote probe staging uses
  `remux_staging_root/probe-inputs`.
- HTTP API and local setup docs describe WebDAV direct play, remux/HLS staging,
  secret references, and preview limitations.
- [Phase 6.5](workstreams/storage-vfs/PHASE6_5_REMOTE_STORAGE_STABILIZATION.md)
  records validation and remaining known limitations.

## Recommended Next Implementation Goal

### M7: Playback Streaming and Remote Hardening

Status: proposed.

Split `playback-streaming` into a dedicated workstream if remote playback is
the next focus. The likely first slice is remote direct response-body
streaming, staging disk budgets and cleanup, richer storage failure mapping,
and multi-library remote configuration.
