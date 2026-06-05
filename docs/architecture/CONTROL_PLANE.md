# Control Plane Architecture

Last updated: 2026-06-05

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
| Durable jobs | Shipped foundation plus schedulable partial, source fingerprint hash contract/summary/internal enqueue/queued planner/single-job executor command, and generic priority policy | ADR 0006; ADR 0053; runtime deepening lanes; durable job queue/resource lane; `docs/workstreams/provider-governance-durable-batch-execution/` | Broader job-kind scheduler migration and source fingerprint hash automatic scheduling remain follow-ons. |
| Runtime supervisor | Shipped resource-accounted foundation | ADR 0019; server runtime deepening; durable job queue/resource lane | Unified trace context and broader scheduler migration remain follow-ons. |
| Resource classes and budgets | Shipped process-local foundation plus source fingerprint hash-to-`disk.scan` mapping | ADR 0005; playback/runtime lanes; durable job queue/resource lane | Continue migrating job kinds onto typed budget-admitted scheduler paths. |
| Tracing/request identity | Partial with HLS and library scan job propagation | diagnostics and playback identity lanes | Continue unified trace context across jobs/FFmpeg/VFS/addons and broader scan entry points. |
| Admin diagnostics | Good partial | Admin API and diagnostics lanes | Safe realtime diagnostics and incident bundles. |
| Crash/fault bundles | Not started | This document | Redacted operator export for hard bugs. |
| Remote access cookbook | Planned | operations/release architecture | Reverse proxy, HTTPS, DDNS, Tailscale, Cloudflare Tunnel guidance. |
| Built-in tunnel provider | Deferred | ADR 0053 | Do not make core depend on a central relay. |
| Endpoint discovery | Not started | This document | LAN/remote endpoint model for clients. |
| API version/error/page contracts | Shipped foundation | ADR 0023; Public/Admin API lanes | Cursor pagination and large-library contracts. |
| HTTP cache/ETag contracts | Narrow shipped partial plus HLS no-store baseline | managed artwork thumbnail/serving lanes; this document | Systematize image, immutable artifact, and catalog cache semantics. |
| N+1/list projection discipline | Partial | catalog projection lanes | API scale tests for large libraries. |

## Workstream Evidence

Use `docs/architecture/WORKSTREAM_LINKS.md#control-plane` as the consolidated
index for runtime, durable job, diagnostics, addon lifecycle, remote access, and
API scale workstreams. Keep this document focused on shared control-plane
capabilities and risks.

## Next Work Lanes

### source-fingerprint-hash-durable-job-contract-and-enqueue

Status: First contract, summary, internal enqueue, queued planner, and
single-job executor command slices shipped as of 2026-06-05.

Goal: Prepare future source fingerprint hash queue/operator integration without
adding execution, API, schema, evidence persistence, or reconciliation
behavior.

Shipped control-plane behavior:

- `JobKind::SourceFingerprintHash` is a persisted durable job kind string;
- `SourceFingerprintHashJobInput` is the only durable input contract for this
  work and carries only Media Library ID, Media Source ID, source scheme, and
  hash mode;
- `disk.scan.source_fingerprint_hash` is the persisted job resource class and
  maps to the existing `disk.scan` runtime budget class.
- `nako-server::app::source_hash` can persist queued source fingerprint hash
  jobs for an existing Media Source after verifying library ownership and
  deriving only the current source scheme from the Source Locator.
- `nako-server::app::source_hash` can prepare a queued source fingerprint hash
  job for future execution by validating the persisted job contract, reloading
  the current Media Source, and rebuilding only an in-memory
  `SourceFingerprintHashRequest` from the current Source Locator.
- `SourceFingerprintHashJobSummary` provides a narrow future `summary_json`
  shape with mode, evidence kind, confidence, stale state, and bytes hashed,
  excluding raw fingerprint/hash material and locator content.
- the internal executor command can claim one explicit source fingerprint hash
  job id through `DurableJobRuntime`, execute VFS-backed hashing, and persist
  the redaction-safe summary JSON.

Follow-ons:

- scan/operator/API triggering beyond the internal app service;
- automatic scheduling under durable leases;
- evidence persistence and redaction-safe Admin diagnostics;
- any automatic duplicate reconciliation policy.

### provider-governance-durable-batch-execution

Status: Closed at
`docs/workstreams/provider-governance-durable-batch-execution/`.

Goal: Move Metadata Candidate Review batch apply from bounded synchronous
Admin confirmation to a durable job-backed workflow with persisted batch
state, progress/status reads, cancellation checkpoints, and redacted per-item
outcomes.

Control-plane requirements:

- create persists a durable job with explicit input (shipped in `PGDBE-030`);
- execution uses `DurableJobRuntime` and runtime resource-class mapping
  (shipped in `PGDBE-040`);
- per-item work calls the existing single-review application authority;
- status reads expose operator-useful, redacted diagnostics;
- no raw `tokio::spawn`, duplicate Provider Mapping executor, Public Client
  API route, related hierarchy application, or Generated Artifact table reuse.

### generated-artifact-provider-mapping-breadth

Status: Closed at
`docs/workstreams/generated-artifact-provider-mapping-breadth/`.

Goal: Add guarded Provider Mapping proposal planning and final apply to the
Generated Artifact Metadata Authority workflow while preserving Admin-only
confirmation, target freshness, redaction, and idempotent outcome behavior.

Control-plane requirements:

- review acceptance must remain staging-only;
- `GAPM-020` read-only planning must not write Provider Mappings;
- `GAPM-030` final Provider Mapping mutation is host-owned, replay-safe, and
  committed with the generated artifact metadata apply outcome;
- `GAPM-040` bulk apply summaries and batch snapshots expose Provider Mapping
  apply/skip/noop counters while reusing the one-artifact apply path instead
  of adding a second provider mapping executor;
- `GAPM-050` Web Admin renders Provider Mapping plan/result facts for single
  and bulk Metadata Authority apply without weakening fallback honesty or
  redaction;
- Admin/Web DTOs must not expose raw payloads, prompts, Source Locators, paths,
  tokens, or secrets.

Follow-ons:

- `docs/workstreams/generated-artifact-apply-operations-repair/` (closed)
- `proposed:provider-identity-mapping-breadth`

### generated-artifact-apply-operations-repair

Status: Closed at
`docs/workstreams/generated-artifact-apply-operations-repair/`.

Goal: Add an Admin recovery workflow for Generated Artifact apply outcomes and
bulk batches so operators can inspect repair-relevant state without raw
internal access.

Shipped control-plane behavior:

- one-artifact apply outcomes have Admin list/detail read paths;
- outcome-only records and bulk batch terminal items feed a read-only recovery
  queue;
- recovery DTOs distinguish replayable success from actionable repair work;
- recovery classification is domain-owned in `nako-core`;
- generated contracts and Web Admin read models are synchronized without raw
  payload, prompt, path, token, or secret exposure.

Follow-ons:

- `docs/workstreams/web-admin-generated-artifact-recovery-ui/` (closed)
- `docs/workstreams/generated-artifact-apply-repair-actions/` (closed)
- `proposed:control-plane-observability-and-trace-context`

### generated-artifact-apply-repair-actions

Status: Closed at
`docs/workstreams/generated-artifact-apply-repair-actions/`.

Goal: Prove the bounded Admin repair action seam for Generated Artifact apply
recovery without adding a blind retry executor or duplicating Metadata
Authority apply logic.

Shipped control-plane behavior:

- recovery-row repair remains preparation-first through the current Metadata
  Authority apply plan;
- live mutation reuses existing single-artifact and bulk apply freshness,
  idempotency, authorization, redaction, and durable audit semantics;
- no backend recovery mutation wrapper or second apply executor is added;
- one-click wrapper and Web copy polish are split as explicit follow-ons.

Follow-ons:

- `proposed:generated-artifact-recovery-one-click-wrapper`
- `proposed:web-generated-artifact-repair-copy-polish`
- `docs/workstreams/metadata-provider-depth-and-precision/` (closed in the
  library pipeline lane)

### generated-artifact-bulk-metadata-apply

Status: Closed at
`docs/workstreams/generated-artifact-bulk-metadata-apply/`.

Goal: Add a guarded Admin bulk apply workflow for accepted metadata Generated
Artifacts while keeping mutation outside the initial read-only plan slice and,
later, outside unbounded HTTP request execution.

Shipped control-plane behavior:

- selection and plan responses must stay redacted;
- confirmed bulk mutation enqueues durable work;
- per-item idempotency and partial-failure state are explicit;
- Web Admin confirms only through live Admin API;
- provider-specific mapping breadth and repair tooling remain follow-ons.

### control-plane-observability-and-trace-context

Status: HTTP request ID first slice shipped as of 2026-06-04; HLS playlist
startup now propagates the typed request ID into playback completion outbox
events; broader cross-runtime propagation remains a follow-on.

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

Shipped behavior:

- root HTTP middleware assigns or normalizes a redaction-safe `x-request-id`;
- `x-request-id` is returned on public, protected, auth-rejected, and
  network/CORS short-circuit responses;
- CORS preflight allows browser clients to send `x-request-id`;
- the typed HTTP trace context is available to handlers through request
  extensions.
- HLS playlist routes convert the typed HTTP trace context into a playback app
  trace context, and HLS `PlaybackSessionFinished` outbox payloads include only
  the normalized `request_id` when the HLS work came from a traced request.
- public and Admin library scan routes convert the typed HTTP trace context
  into the durable job trace context before enqueueing `disk.scan` work, and
  completed `LibraryScanned` outbox payloads include only the normalized
  `request_id` when one was provided.

Exit criteria:

- HTTP library scan routes and additional scheduler-originated scan paths emit
  correlated trace IDs;
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
- generic durable job priority is persisted with queued work, inherited by
  retries, used by lease claiming, and bounded by a starvation guard so aged
  lower-priority work can still run;
- broader job-kind scheduler migration remains a follow-on after the generic
  priority policy baseline.

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

Current shipped artifact-cache baseline:

- HLS playlist and HLS segment responses are explicitly `Cache-Control:
  no-store` because they are session/ticket scoped playback artifacts. Immutable
  segment caching, ETags, and conditional GET behavior remain follow-ons until
  token-aware cache keys and artifact invalidation are specified.
- Direct Play and Remux media byte GET/HEAD/range responses are explicitly
  `Cache-Control: no-store` because they are authenticated or short-lived-ticket
  scoped media transport responses. Media-byte ETags, conditional GET, and
  shared-cache behavior remain follow-ons.
- Authenticated selected artwork image GET/HEAD responses now use
  `Cache-Control: private, max-age=86400` with existing safe ETags. Matching
  `If-None-Match` requests, including exact, weak, validator-list, and wildcard
  forms, return `304 Not Modified` with ETag/cache headers, and
  metadata-derived selected artwork ETag preflight can short-circuit matching
  304 responses after auth and library access checks. Immutable/public shared
  caching, CDN semantics, and selected-artwork invalidation policy remain
  follow-ons.

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
