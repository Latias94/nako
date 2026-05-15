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

## Active Goal

### M5: Extension and Automation Surface

Status: active, completed through M5.1 event outbox foundation.

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

## Recommended Next Implementation Goal

### M5.2: Webhook Delivery Worker

Status: proposed.

Implement webhook endpoint configuration, delivery attempts, bounded dispatch,
retry/backoff state, signing policy, safe error mapping, and delivery
inspection. Delivery should consume the event outbox; domain workflows must not
call webhook HTTP endpoints inline.

## Later Goals

### M6: Remote Storage Preview

Implement one limited remote storage backend and prove scan/probe/playback can
work through `taru-vfs` without local-path assumptions.
