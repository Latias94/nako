# Nako Architecture

Last updated: 2026-05-29

Nako is a self-hosted media server backend. The long-term target is a
Jellyfin/Plex-class system that remains self-hostable, inspectable, and easy to
extend without copying Jellyfin/Plex internals or accepting a native plugin ABI
too early.

This document is the architecture map. `CONTEXT.md` owns vocabulary, ADRs own
durable decisions, and workstreams own task-level execution evidence.

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
| Clients | Admin Web, Media Web foundation, Android/shared client core lanes. | Mixed | Public media client parity, player UX, TV/casting clients. |
| Deployment | Self-hosted docs, backup/restore, release gates. | Good | HTTPS/tunnel/reverse proxy recipes, observability, larger install profiles. |

## Playback And Transcode Map

The target playback decision flow is:

```text
MediaSource + MediaProbeResult + ClientPlaybackCapabilities + Policy
  -> PlaybackRenditionPlan
  -> Direct Play | Remux | HLS Transcode
  -> typed request identity
  -> runtime session and artifact manifest
  -> safe Public/Renderer transport URL
```

Current executable coverage includes:

- direct byte-range playback and HEAD preflight;
- FFmpeg copy-remux planning and session reuse;
- HLS single-variant MPEG-TS and fMP4 output;
- adaptive HLS fMP4 ladders;
- source-aware ladder dimensions and no-audio adaptive variants;
- selected audio stream mapping;
- selected subtitle WebVTT sidecars and master playlist `TYPE=SUBTITLES`;
- generated audio sidecars and master playlist `TYPE=AUDIO`;
- FFmpeg hardware planning for VAAPI, NVENC, QuickSync, AMF, and
  VideoToolbox-shaped policies;
- startup hardware readiness, CPU fallback, session cleanup, and safe failure
  redaction.

Important gaps before Jellyfin/Plex-class playback:

- HDR to SDR tone mapping and color pipeline policy;
- ASS/SSA subtitle burn-in and exact subtitle rendering strategy;
- seek/restart model for long-running HLS sessions;
- bandwidth-aware ABR and variant pruning;
- GPU decode/encode concurrency scheduling and queueing;
- richer device capability profiles for browsers, mobile apps, TV clients,
  Chromecast, DLNA, and AirPlay;
- optional DASH/CMAF and LL-HLS lanes after HLS behavior is stable.

## Metadata And Library Map

The catalog target is:

```text
Source Locator + file name/path facts + ffprobe facts + NFO + providers + addons
  -> Local Inference / Provider Mapping / Field Locks
  -> Canonical Metadata
  -> Catalog Graph
  -> Search Projection
```

Current executable coverage includes local media discovery, metadata merge
policy, NFO authority, TMDB-shaped provider work, canonical catalog projection,
and search hydration. Remaining product pressure is provider breadth:
TMDB series/season/episode depth, Douban, Bangumi, anime-specific structure,
artwork lifecycle polish, and operator-visible diagnostics.

## Extension Map

Nako's extension target is out-of-process:

```text
Addon Package / Addon Suite
  -> Addon Sidecar
  -> scoped Addon Token + Grant
  -> Addon Resource / Addon Task / Event Subscription
  -> host-owned policy and persistence
```

This keeps self-hosted extension power without giving arbitrary addon code
direct database, raw library path, or in-process memory access.

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

Progress trackers:

- `docs/ROADMAP.md`
- `docs/GOALS.md`
- `docs/workstreams/README.md`

## Update Policy

Update this document when a lane changes Nako's system map, not for every task.
Use ADRs for durable decisions, workstreams for execution, and `ROADMAP.md` for
phase-level completion and future breadth.
