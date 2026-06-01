# Control Plane Architecture

Last updated: 2026-06-01

This document maps Nako's application control plane: the cross-cutting systems
that keep the media data plane safe, observable, extensible, and operable.

The media data plane moves bytes, probes sources, plans playback, runs FFmpeg,
and serves artifacts. The control plane decides who may do that work, when it
may run, how it is supervised, how it is diagnosed, and how clients consume it
at scale.

## Target Chain

```text
HTTP / Addon / Event / Scheduled Request
  -> Authenticated Principal or Trusted Internal Actor
  -> Policy and Capability Check
  -> Request Identity and Trace Context
  -> Durable Job or Supervised Runtime Task
  -> Resource Class and Budget
  -> VFS / FFmpeg / Provider / Addon Sidecar / Database Work
  -> Redacted Diagnostics, Events, Cache Headers, or API Response
```

Remote access and client scale use the same control-plane boundary:

```text
Deployment Endpoint Config
  -> trusted proxy and public URL policy
  -> LAN / remote endpoint discovery
  -> playback ticket or API auth
  -> cacheable or paginated client response
```

## Progress Matrix

| Capability | Status | Authority | Next Lane |
| --- | --- | --- | --- |
| Control-plane boundary | New architecture baseline | ADR 0053; this document | Keep future cross-cutting work mapped here. |
| HTTP addon protocol | Shipped foundation | ADR 0003; ADR 0015; ADR 0020 | Addon Manager lifecycle is still deferred. |
| Addon capability and token scopes | Shipped foundation | addon token/grant workstreams | Stronger hosted surface and route policy. |
| Addon process supervision | Deferred | ADR 0020; ADR 0053 | `addon-manager-process-lifecycle`. |
| Durable jobs | Shipped foundation plus schedulable partial | ADR 0006; runtime deepening lanes; durable job queue/resource lane | `proposed:durable-job-priority-policy-and-scheduler-migration` before playback-affecting work needs priority. |
| Runtime supervisor | Shipped resource-accounted foundation | ADR 0019; server runtime deepening; durable job queue/resource lane | Unified trace context and broader scheduler migration remain follow-ons. |
| Resource classes and budgets | Shipped process-local foundation | ADR 0005; playback/runtime lanes; durable job queue/resource lane | Broader job-kind scheduler migration after priority policy is concrete. |
| Tracing/request identity | Partial | diagnostics and playback identity lanes | Unified trace context across HTTP/jobs/FFmpeg/VFS/addons. |
| Admin diagnostics | Good partial | Admin API and diagnostics lanes | Safe realtime diagnostics and incident bundles. |
| Crash/fault bundles | Not started | This document | Redacted operator export for hard bugs. |
| Remote access cookbook | Planned | operations/release architecture | Reverse proxy, HTTPS, DDNS, Tailscale, Cloudflare Tunnel guidance. |
| Built-in tunnel provider | Deferred | ADR 0053 | Do not make core depend on a central relay. |
| Endpoint discovery | Not started | This document | LAN/remote endpoint model for clients. |
| API version/error/page contracts | Shipped foundation | ADR 0023; Public/Admin API lanes | Cursor pagination and large-library contracts. |
| HTTP cache/ETag contracts | Narrow shipped partial | managed artwork thumbnail/serving lanes; this document | Systematize image, artifact, and catalog cache semantics. |
| N+1/list projection discipline | Partial | catalog projection lanes | API scale tests for large libraries. |

## Workstream Evidence

Use `docs/architecture/WORKSTREAM_LINKS.md#control-plane` as the consolidated
index for runtime, durable job, diagnostics, addon lifecycle, remote access, and
API scale workstreams. Keep this document focused on shared control-plane
capabilities and risks.

## Next Work Lanes

### control-plane-observability-and-trace-context

Goal: Make operator-visible diagnostics and developer traces follow one request
from API entry through policy, database, VFS, FFmpeg, addon, and job runtime
work.

Scope:

- request ID and trace context propagation;
- `tracing` span policy for HTTP, durable jobs, VFS, FFmpeg, and addons;
- redacted FFmpeg command/session diagnostics;
- Tokio task/runtime diagnostics;
- safe incident bundle export for operators;
- no opt-out-hostile telemetry from self-hosted installs.

Exit criteria:

- playback and library scan paths emit correlated trace IDs;
- Admin diagnostics can show safe recent failures without raw paths, tokens, or
  provider payloads;
- tests cover redaction for diagnostic DTOs.

### durable-job-queue-and-resource-classes

Status: Closed workstream at
`docs/workstreams/durable-job-queue-and-resource-classes/`.

Goal: Upgrade durable background work from persisted job records plus helper
spawns into a schedulable queue with resource class accounting, retry/backoff,
queue pressure diagnostics, and the first typed budget-admitted scheduler path.

Scope:

- job state machine and lease rules;
- retry and backoff policy;
- cancellation and startup recovery semantics;
- resource classes for scan, metadata, artwork, subtitles, playback-adjacent
  artifacts, offline sync, addon, and webhook work;
- queue pressure diagnostics.

Closeout:

- resource classes and budgets are centralized in the process-local runtime;
- library scan jobs have the first typed budget-admitted scheduler path;
- retry/backoff rows and queue pressure diagnostics are persisted and redacted;
- priority policy is split to
  `proposed:durable-job-priority-policy-and-scheduler-migration`.

### api-scale-and-cache-contracts

Goal: Prevent large libraries from turning client browse, search, artwork, and
admin pages into unbounded JSON or repeated SQL loops.

Scope:

- cursor pagination rules for stable browse/search orderings;
- projection-backed list endpoints;
- API response budget guidance;
- `Cache-Control`, `ETag`, and immutable artifact headers;
- image/artwork derivative cache behavior;
- N+1 query regression tests for common list/detail surfaces.

Exit criteria:

- no public browse route needs unbounded list responses;
- image/artifact routes have explicit cache semantics;
- large-library tests cover query and response-size budgets.

### self-hosted-remote-access-and-endpoint-discovery

Goal: Make self-hosted remote access predictable without making Nako core a
tunnel service.

Scope:

- trusted reverse proxy headers;
- public base URL and LAN URL configuration;
- HTTPS and playback ticket caveats;
- DDNS, Caddy/Nginx, Tailscale, and Cloudflare Tunnel cookbook;
- client endpoint selection model for LAN versus remote.

Exit criteria:

- deployment docs explain supported remote access shapes;
- server diagnostics report sanitized endpoint configuration;
- clients can reason about LAN and remote endpoints without changing playback
  contracts.

### addon-manager-process-lifecycle

Goal: Provide optional host-side lifecycle management for addon sidecars without
weakening the out-of-process addon trust boundary.

Scope:

- installed addon package inventory;
- process start/stop/restart policy;
- health checks and log pointers;
- port allocation and environment rendering;
- package signature/update/rollback policy;
- explicit non-goal for in-process native plugin ABI.

This lane should wait until addon protocol, permissions, and official addon
catalog behavior are stable enough to manage packages safely.

## Risk Register

### Background Work Can Starve Playback

Scan, trickplay, artwork, AI indexing, and offline sync can consume the same
CPU, GPU, disk, and network resources as playback. The queue needs resource
classes and priority before these workflows become broad.

### Diagnostics Can Leak Host Secrets

Control-plane data often contains local paths, source locators, FFmpeg command
lines, bearer tokens, addon credentials, provider payloads, and private titles.
Diagnostics must be useful and redacted at the same time.

### Built-In Tunnels Can Become A Central Service Commitment

Self-hosted users need remote access, but a first-party relay changes Nako's
operational, security, cost, and abuse model. Prefer explicit reverse proxy and
third-party tunnel compatibility until a dedicated product decision supersedes
this boundary.

### Addon Manager Can Blur The Trust Boundary

Starting and updating sidecar processes is not the same as trusting their code.
Even managed sidecars must keep scoped tokens, library grants, host-owned
storage, and host-owned persistence boundaries.

### Cache Correctness Is Access Control

HTTP caches, ETags, and artwork derivatives must not serve data across users,
libraries, or permission changes. Cache keys and headers need to include the
right public/private boundary.

## Agent Notes

When a future workstream adds a cross-cutting runtime, queue, diagnostics,
remote access, addon lifecycle, or API scale concern, update this document and
ADR 0053 references before changing code. Do not hide new background work in a
raw `tokio::spawn` path when it needs durable state or resource policy.
