# Nako Architecture

Last updated: 2026-05-29

Nako is a self-hosted media server backend. The long-term target is a
Jellyfin/Plex-class system that remains self-hostable, inspectable, and easy to
extend without copying Jellyfin/Plex internals or accepting a native plugin ABI
too early.

This document is the architecture map. `CONTEXT.md` owns vocabulary, ADRs own
durable decisions, and workstreams own task-level execution evidence.

Detailed capability maps live in `docs/architecture/`. Workstream execution
evidence is linked from those deep dives and from
`docs/architecture/WORKSTREAM_LINKS.md`.

Deep dives:

- `docs/architecture/PLAYBACK.md`: playback capability progress,
  workstream/ADR links, parallel lanes, and risk register.
- `docs/architecture/STORAGE_VFS.md`: storage/VFS resilience, source identity,
  remote staging, and mount-risk map.
- `docs/architecture/LIBRARY_PIPELINE.md`: scan, watcher, probe, metadata,
  artwork, and addon-assisted intake map.
- `docs/architecture/STATE_ACCESS.md`: database, playback state, identity,
  access, and write-pressure map.
- `docs/architecture/REALTIME_SYNC.md`: realtime client updates, event
  boundary, and offline sync map.
- `docs/architecture/OPERATIONS_RELEASE.md`: deployment, release, diagnostics,
  backup, and packaging map.
- `docs/architecture/CONTROL_PLANE.md`: addon lifecycle, observability, durable
  jobs, remote access, API scale, and cache-contract map.
- `docs/architecture/WORKSTREAM_LINKS.md`: architecture capability to
  workstream evidence index.

## North Star

Nako should be able to:

- index large local and remote media libraries;
- reconcile local, NFO, provider, and addon metadata into canonical catalog
  state;
- choose Direct Play, Remux, or Transcode from explicit source/client/user
  facts;
- stream through safe browser, renderer, mobile, and future TV clients;
- use FFmpeg hardware acceleration through typed planning, not ad hoc command
  strings;
- expose addon, webhook, and automation surfaces without trusting arbitrary
  in-process code;
- keep operator deployment simple by default while allowing larger installs to
  move to PostgreSQL and external services.

## Architecture Principles

- **Direct Play first.** Transcode is a fallback, not the default media path.
- **Planner before runtime.** Playback, transcode, metadata, and addon decisions
  should be typed plans before they become processes, SQL writes, URLs, or
  external calls.
- **Manifest-backed artifacts.** Nako should not publish URLs for playback or
  managed artifacts until a typed manifest can prove that the artifact is
  generated, servable, and safe to expose.
- **FFmpeg CLI first.** Rust owns planning, process supervision, lifecycle,
  policy, and serving. FFmpeg/ffprobe own media decoding, encoding, muxing,
  probing, and HLS/DASH-like media output until a dedicated engine lane proves
  otherwise.
- **Resource budgets are product behavior.** CPU, GPU, disk, network, staging,
  addon, webhook, and scan work must remain bounded and observable.
- **Local authority remains explicit.** NFO, sidecars, user edits, and field
  locks are not provider accidents; they are first-class self-hosted behavior.
- **Addons are out-of-process.** Addon sidecars interact through scoped HTTP
  APIs and tokens, not an in-process plugin ABI.
- **Control plane work is explicit.** Durable jobs, runtime supervision,
  diagnostics, remote access, addon lifecycle, and API scale belong to shared
  control-plane boundaries, not hidden per-feature helpers.

## System Map

| Area | Current Shape | Maturity | Next Pressure |
| --- | --- | --- | --- |
| Domain model | `nako-core` owns media/library/catalog/user/playback records and repository traits. | Strong foundation | Broader music/photo/document domain breadth. |
| Persistence | SQLite default with PostgreSQL-ready boundaries and migration policy. | Strong foundation | Production Postgres parity for newer feature tables. |
| Storage/VFS | Local and remote storage boundaries with staging and remote playback policy. | Good | Cache policy, remote write/import promotion, network tunnel deployment stories. |
| Scan | Durable scan state, source tombstones, local inference, and library ingestion seams. | Good | File watcher/incremental scan productization and large-library scheduling. |
| Metadata | TMDB/NFO/local authority, provider payload boundaries, catalog graph, search projection. | Good first video slice | Douban, Bangumi, series/anime breadth, provider diagnostics, richer artwork. |
| Playback planning | Typed rendition planning, policy, renderer targets, capability profiles, request identity. | Good | Device profiles, precise client codec/container/subtitle/HDR capability reporting. |
| Transcode runtime | FFmpeg CLI planning, remux/HLS sessions, hardware policy, fMP4/adaptive/source-aware HLS, subtitle/audio sidecars. | Advancing rapidly | HDR tone mapping, ASS burn-in, seek restart model, ABR refinement, GPU resource scheduling. |
| Browser transport | Short-lived playback tickets, safe HLS/remux/direct URLs, bearer redaction. | Good | Player integration, HLS.js/Shaka behavior, cross-device resume polish. |
| User state | Durable session auth, library access, playback progress, continue watching. | Good | Multi-device conflict semantics and richer active-session controls. |
| Addons/automation | HTTP addon sidecars, grants, tasks, events, official addon catalog direction. | Good foundation | Marketplace/install guidance, official provider breadth, external resource actions. |
| Control plane | Auth, runtime supervision, durable jobs, diagnostics, addon mediation, remote access guidance, and API scale contracts are mapped as shared infrastructure. | Good foundation | Unified trace context, job queue priority/retry, endpoint discovery, ETag/cache contracts. |
| Clients | Admin Web, Media Web foundation, Android/shared client core lanes. | Mixed | Public media client parity, player UX, TV/casting clients. |
| Deployment | Self-hosted docs, backup/restore, release gates. | Good | HTTPS/tunnel/reverse proxy recipes, observability, larger install profiles. |

## Documents Of Record

Core ADRs:

- `docs/adr/0001-modular-monolith-rust-workspace.md`
- `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
- `docs/adr/0008-nfo-as-local-metadata-boundary.md`
- `docs/adr/0012-durable-scan-state-and-source-tombstones.md`
- `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md`
- `docs/adr/0021-video-first-media-server-domain-model.md`
- `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`
- `docs/adr/0044-playback-capability-profile-planner.md`
- `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
- `docs/adr/0049-source-aware-transcode-runtime.md`
- `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`
- `docs/adr/0053-application-control-plane-boundary.md`

Progress trackers:

- `docs/architecture/PLAYBACK.md`
- `docs/architecture/STORAGE_VFS.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/STATE_ACCESS.md`
- `docs/architecture/REALTIME_SYNC.md`
- `docs/architecture/OPERATIONS_RELEASE.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/ROADMAP.md`
- `docs/GOALS.md`
- `docs/workstreams/README.md`

## Update Policy

Update this document when a lane changes Nako's system map, not for every task.
Use ADRs for durable decisions, workstreams for execution, and `ROADMAP.md` for
phase-level completion and future breadth.
