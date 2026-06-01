# Nako MVP

Status: Initial cut
Last updated: 2026-06-01

## MVP Statement

The first Nako MVP is a video-first, self-hosted, single-admin media server
release that lets an operator install Nako, configure a media library, scan
media, review metadata authority, browse media, play media through a browser
client path, diagnose playback/storage problems, and manually register external
Addon Sidecars.

The MVP should feel like a real private media home, not a demo of disconnected
subsystems.

## Primary User

A self-hosted operator running Nako for themselves or a small trusted group.
The operator values local control, transparent diagnostics, and extensibility,
but accepts that the first release is not a full Jellyfin/Plex replacement.

## Required User Journey

1. Install and start Nako with a documented default configuration.
2. Authenticate as the initial administrator.
3. Configure at least one Media Library.
4. Scan the library and see source, probe, metadata, and storage outcomes.
5. Browse Media Items through the accepted web/client surface.
6. Start playback through Direct Play, Remux, or HLS Transcode when needed.
7. See clear redacted diagnostics when playback, scan, storage, metadata, or
   FFmpeg behavior fails.
8. Register and inspect an external Addon Sidecar without giving it in-process
   trust.
9. Configure remote access through documented endpoints, reverse proxies, or
   third-party tunnel providers.

## P0 Scope

P0 is required for the first MVP release.

| Area | MVP requirement |
| --- | --- |
| Install and config | Documented local or container startup, SQLite default, health check, and safe config examples. |
| Access | Single-Admin Mode works; the domain language still preserves User, Role, and Library Access. |
| Media library | At least one local library path works end-to-end; existing remote storage behavior may ship if already stable but should not expand the release cut. |
| Scan and source state | Scan, source records, probe facts, tombstones, and redacted failures are visible enough to troubleshoot. |
| Metadata | Local inference, NFO, and at least one provider-backed path can produce Canonical Metadata without hiding authority. |
| Browse | A web/client path can list/search/detail Media Items through the Public Client API boundary. |
| Playback | Direct Play, Remux, and HLS Transcode are usable with CPU fallback and clear failure behavior. |
| Transcode diagnostics | FFmpeg/ffprobe presence, hardware capability report, fallback/fail-fast policy, and safe errors are visible to operators. |
| Admin diagnostics | Admin Web or Admin API exposes storage health, scan/job state, playback sessions, and runtime readiness without leaking secrets or raw paths. |
| Addon sidecar foundation | Manual Addon Sidecar registration, health check, scoped token/grant model, and resource-call diagnostics are available. |
| Network access | Remote Access Endpoint configuration and operator cookbook exist for reverse proxy, HTTPS, DDNS, Tailscale, or Cloudflare Tunnel. |
| Release evidence | Focused gates prove install/config docs, scan, metadata, playback, diagnostics, redaction, and release packaging behavior. |

## P1 Scope

P1 follows the MVP unless a P0 blocker proves it must move earlier.

- Web Admin apply workflow polish after backend Generated Artifact metadata
  apply routes.
- Desktop playback strategy spike and packaging decision.
- Playback release hardware matrix and optional vendor smoke evidence.
- Watcher/debounce productization.
- API cache/ETag contracts for large-library scale.
- Client realtime gateway for scan/playback/catalog updates.
- Official addon catalog breadth and cross-repo smoke providers.
- Provider breadth and diagnostics for Douban, Bangumi, and richer series/anime
  behavior.

## P2 Scope

P2 is explicitly post-MVP.

- Built-in tunnel provider or first-party relay.
- Addon Manager installation, updates, process supervision, and package
  rollback.
- Mobile, TV, or desktop-native production clients.
- Remote transcode workers.
- LL-HLS/CMAF, DASH, DRM, and offline sync/download-to-go.
- Recommendation engine, AI model runtime, vector search, and broad automation
  mutation without review.
- Jellyfin plugin compatibility or native in-process plugin ABI.

## Release Rule

A feature can be advanced before MVP only when it either:

- unblocks the required user journey;
- reduces a P0 release risk;
- closes an active workstream needed for release convergence;
- or prevents a costly architecture mistake in a P0 path.

Everything else should be split or deferred.
