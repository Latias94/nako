# 0053: Treat The Application Control Plane As A First-Class Boundary

## Status

Accepted

## Status Note

Accepted as an architecture baseline for future work. Existing implementation
already has pieces of this boundary through addon sidecars, durable jobs,
runtime supervision, diagnostics, authentication, playback tickets, and
versioned APIs. Follow-on workstreams should deepen the missing pieces without
turning them into ad hoc per-feature infrastructure.

## Context

Nako has made fast progress in media-server data-plane areas: storage/VFS,
library scanning, metadata, playback planning, HLS, FFmpeg process execution,
and managed artifacts. As the project grows toward a Jellyfin/Plex-class
self-hosted application, the late-stage refactor risk moves up one layer.

The risky cross-cutting systems are:

- addon lifecycle and process isolation;
- observability, tracing, diagnostics, and crash/fault evidence;
- durable background jobs, retry, cancellation, and resource priority;
- remote access through reverse proxies, tunnels, and LAN/remote endpoint
  selection;
- API performance, pagination, projection, and cache contracts.

Nako already has related ADRs:

- ADR 0003 chooses HTTP addons before in-process plugins.
- ADR 0005 requires bounded async pipelines and resource budgets.
- ADR 0006 requires persisted job inputs and explicit retry policy.
- ADR 0014 defines a durable event outbox.
- ADR 0015 defines capability-scoped addons and automation providers.
- ADR 0019 defines explicit runtime supervisors.
- ADR 0020 defines sidecar addons with scoped Nako API access.
- ADR 0023 stabilizes public API versions and error envelopes.
- ADR 0036 defines short-lived browser playback tickets.
- ADR 0052 defines the FFmpeg CLI-first HLS runtime boundary.

What is missing is a single control-plane boundary that tells future agents
where these concerns belong and what must not be rebuilt inside individual
features.

## Decision

Treat Nako's application control plane as a first-class architecture boundary,
distinct from the media data plane.

The media data plane moves bytes, probes media, plans playback, runs FFmpeg,
serves streams, and materializes artifacts. The control plane owns policy,
identity, authorization, durable work, supervision, resource accounting,
diagnostics, endpoint configuration, addon mediation, and API scale contracts.

Future control-plane work must follow these rules:

- Addons remain out-of-process by default. Nako may later manage sidecar
  process lifecycle, package installation, health checks, logs, and updates, but
  managed sidecars still communicate through the Nako Addon Protocol, scoped
  tokens, accepted permissions, and library grants. A managed addon is not an
  in-process plugin.
- Long-running or important background work must enter a durable job or
  supervised runtime boundary. Raw `tokio::spawn` is acceptable only for small
  request-local or explicitly disposable work.
- Job execution must carry a resource class and policy when it can compete with
  playback, storage, provider, addon, webhook, or metadata work.
- Request identity and trace context should propagate across HTTP handlers,
  jobs, VFS calls, FFmpeg processes, addon calls, webhook delivery, and public
  or admin responses where useful.
- Diagnostics must be operator-useful and redacted. They must not expose raw
  library paths, source locators, cache URIs, FFmpeg command lines containing
  secrets, bearer tokens, addon credentials, provider payloads, or raw stderr
  where it may contain host-sensitive data.
- Nako should support remote access through explicit endpoint configuration,
  trusted proxy handling, HTTPS guidance, playback-ticket boundaries, and
  third-party tunnel compatibility. Nako core should not depend on a
  first-party central relay or built-in tunnel provider unless a future ADR
  changes the product and operational commitment.
- Public and Admin APIs must remain scalable for large libraries. New browse,
  search, image, artifact, and admin list surfaces should use bounded
  pagination, projection-backed reads, cache headers where safe, and tests that
  prevent unbounded list responses or avoidable N+1 query behavior.
- Self-hosted telemetry must remain explicit. Nako may expose local diagnostics
  and operator-exported incident bundles, but it should not silently phone home.

## Consequences

- Future implementation lanes can share a common mental model for addon
  manager, job queue, observability, remote access, and API scale work.
- Playback, metadata, artwork, offline sync, and AI/vector workflows should not
  each invent their own queue, retry, trace, or resource policy.
- The Admin API and future Admin Web can grow from safe diagnostics and runtime
  summaries rather than raw internal records.
- Built-in tunnel, marketplace, package update, and process supervision work is
  recognized as valuable but not accidentally smuggled into the server core.
- Crate boundaries may need follow-on review. The ADR does not require a
  `nako-control-plane` crate, but it does create pressure to check whether
  jobs, observability, runtime supervision, and API scale contracts are
  currently grouped well.

## Alternatives Considered

- Let each feature own its own background tasks, diagnostics, and retry logic.
  Rejected because it creates inconsistent failure behavior and makes playback
  resource protection harder.
- Embed native or in-process plugins for maximum power. Rejected by ADR 0003
  and ADR 0020 because Rust ABI, crash isolation, trust, and versioning risks
  are too high.
- Build a first-party tunnel/relay as part of the core server. Deferred because
  it creates a central service commitment and a materially different threat,
  cost, abuse, and support model.
- Use durable outbox rows for all realtime UI updates. Rejected as a default:
  durable external delivery and ephemeral client state updates have different
  latency and recovery needs.
- Rely on clients to handle huge unbounded JSON responses. Rejected because TV,
  mobile, and low-power browser clients need server-side pagination,
  projections, and cache contracts.

## Related Workstreams

- `docs/workstreams/addons-automation/`
- `docs/workstreams/server-runtime-deepening/`
- `docs/workstreams/durable-job-recovery/`
- `docs/workstreams/admin-web-console/`
- `docs/workstreams/public-api-contract/`
- `docs/workstreams/playback-streaming/`
- `docs/workstreams/media-server-architecture-progress-map/`
