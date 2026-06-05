# Storage And VFS Architecture

Last updated: 2026-06-05

This document maps Nako's storage and VFS architecture for agents working on
scan, probe, playback, imports, sidecar writes, and remote storage.

## Target Chain

```text
Library Config
  -> StorageBackend / VFS capabilities
  -> Source Locator
  -> Source Fingerprint / duplicate evidence
  -> probe/scan/playback staging
  -> manifest-backed cleanup and diagnostics
```

Storage is fallible product behavior. Local disks, NAS mounts, WebDAV, SMB/NFS,
and rclone-like mounts can be slow, stale, or unavailable.

## Progress Matrix

| Capability | Status | Authority | Next Lane |
| --- | --- | --- | --- |
| Local storage backend | Shipped | `docs/adr/0002-internal-vfs-before-os-mounting.md` | Keep local behavior as the compatibility baseline. |
| Remote storage boundary | Shipped durable health foundation | `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md`; `docs/workstreams/storage-vfs-resilience-and-source-identity/`; `docs/workstreams/remote-storage-health-and-circuit-breaker/`; `.trellis/tasks/archive/2026-06/06-02-01d-hls-artifact-io-pressure-enforcement/` | Open follow-ons for cache repair, fingerprint hash queue/operator integration, scan scheduling, or PostgreSQL runtime harness work. |
| WebDAV read path | Partial | `docs/workstreams/storage-vfs/`; remote storage lanes | Harden retries, cache, and operator diagnostics. |
| Source locator | Shipped foundation | `CONTEXT.md`; `docs/workstreams/storage-vfs-resilience-and-source-identity/` | Watcher/debounce productization and repair workflows. |
| Source fingerprint | Shipped escalation policy, hash execution kernel, scheduling diagnostic planner, durable job contract, job summary contract, internal enqueue seam, queued execution planner, single-job executor command, scheduler integration, and evidence persistence | `CONTEXT.md`; `docs/workstreams/storage-vfs-resilience-and-source-identity/`; `.trellis/tasks/archive/2026-06/06-04-06-04-source-fingerprint-escalation-policy-first-slice/`; `.trellis/tasks/archive/2026-06/06-05-06-05-source-fingerprint-hash-execution-first-slice/`; `crates/nako-core/src/job.rs`; `crates/nako-library/src/source_hash.rs`; `crates/nako-server/src/app/source_hash.rs`; `crates/nako-server/src/app/jobs.rs`; `crates/nako-server/src/app/runtime.rs` | Operator/Admin/Public API triggering and automatic reconciliation remain follow-ons. |
| Remote probe staging | Shipped foundation | `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md`; `docs/workstreams/storage-vfs-resilience-and-source-identity/` | Per-backend staging budgets and diagnostics. |
| Remote FFmpeg input staging | Shipped foundation | `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md` | Per-backend staging budgets and diagnostics. |
| VFS cache | Shipped diagnostics foundation, action preview, latest-failure refresh, action plan, target-scoped preview, and selected-target refresh execution | `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md`; `docs/workstreams/storage-vfs-resilience-and-source-identity/`; `.trellis/tasks/06-04-06-04-vfs-cache-repair-action-preview-first-slice/`; `.trellis/tasks/06-04-vfs-cache-repair-operator-actions/`; `.trellis/tasks/06-04-vfs-cache-uri-scoped-previews/`; `.trellis/tasks/06-05-vfs-cache-repair-executable-refresh-action/` | Durable repair queues and broader non-destructive remediation planning remain follow-ons. |
| Library file writes | Partial | addon/library-file-write and NFO workstreams | Capability-specific write/link/backup policy. |
| Mount hang protection | Shipped durable circuit foundation | `docs/workstreams/storage-vfs-resilience-and-source-identity/`; `docs/workstreams/remote-storage-health-and-circuit-breaker/` | OS-level mount stalls still need bounded adapters and operator guidance; do not claim syscall preemption. |

## Workstream Evidence

Use `docs/architecture/WORKSTREAM_LINKS.md#storage-and-vfs` as the consolidated
index for storage/VFS workstreams. Keep this document focused on capability
state and risk, not copied task evidence.

## Completed Work Lanes

### remote-storage-health-and-circuit-breaker

Status: Closed as of 2026-05-31.

Shipped:

- durable **Storage Backend Health** records and repository parity;
- runtime **Storage Circuit Breaker** admission for bounded storage work;
- redaction-safe Admin diagnostics and operator reset;
- generated Admin TypeScript contract refresh for the new DTOs and routes.

Follow-ons remain separate: cache repair, source fingerprint hash
queue/operator integration, playback artifact I/O scheduling, scan scheduling,
and PostgreSQL runtime harness evidence.

### storage-vfs-resilience-and-source-identity

Status: Completed as of 2026-05-30.

Shipped:

- layered redaction-safe **Source Fingerprint** evidence;
- strong-evidence move/rename reconciliation;
- reviewable **Source Duplicate Relationship** records;
- redaction-safe storage failure classification;
- bounded process-local read/probe/stage backoff;
- Admin diagnostics for catalog governance, VFS cache/staging cleanup pressure,
  and storage backend health.

### source-fingerprint-hash-execution-first-slice

Status: Minimal execution kernel shipped as of 2026-06-05.

Shipped:

- `nako-library::source_hash` executes explicit partial and full source
  fingerprint hash modes through VFS;
- partial mode reads a configured prefix range and returns redaction-safe
  `BackendFingerprint` evidence;
- full mode streams the complete object with `stream_range(uri, None)` and
  returns redaction-safe `ContentHash` evidence;
- `LocalFsBackend` now supports streaming range reads and explicit range reads
  without forcing callers to load the whole file first.

Boundaries:

- no scan scheduling, operator queue, Admin/Public API, persistence, duplicate
  relationship mutation, or automatic Media Source merge behavior was added.

### source-fingerprint-hash-scheduling-diagnostics-first-slice

Status: Minimal scheduling diagnostic planner shipped as of 2026-06-05.

Shipped:

- `nako-library::source_hash` can map an advisory
  `SourceFingerprintEscalationDecision` and opt-in scheduling policy into an
  optional `SourceFingerprintHashRequest`;
- disabled scheduling and `none` escalation remain diagnostic-only;
- partial escalation selects the configured prefix-length
  `SourceFingerprintHashMode::Partial`, while full escalation selects
  `SourceFingerprintHashMode::Full`;
- diagnostics expose only source scheme, decision facts, schedule state, and
  selected mode, keeping raw source locators and local paths out of operator
  surfaces.

Boundaries:

- no VFS read, durable queue, Admin/Public API, persistence, schema migration,
  duplicate relationship mutation, or automatic Media Source merge behavior was
  added.

### source-fingerprint-hash-durable-job-contract-first-slice

Status: Durable job contract shipped as of 2026-06-05.

Shipped:

- `JobKind::SourceFingerprintHash` round-trips through the persisted durable job
  kind string `source_fingerprint_hash`;
- `nako-library::source_hash` exposes
  `SourceFingerprintHashJobInput` for future persisted work with only Media
  Library ID, Media Source ID, source scheme, and partial/full hash mode;
- the future persisted job resource class
  `disk.scan.source_fingerprint_hash` maps to the existing `disk.scan` runtime
  budget class in `nako-server`.

Boundaries:

- no durable enqueue service, scheduler/executor, VFS read, Admin/Public API,
  schema migration, evidence persistence, duplicate relationship mutation, or
  automatic Media Source merge behavior was added.
- Raw `StorageUri`, Source Locator, local path, backend URL, credential, etag,
  fingerprint, and hash material remain outside durable job input and
  diagnostic surfaces.

### source-fingerprint-hash-enqueue-service-first-slice

Status: Internal enqueue seam shipped as of 2026-06-05.

Shipped:

- `nako-server::app::source_hash` can enqueue
  `JobKind::SourceFingerprintHash` jobs for an existing Media Source;
- the service loads the current Media Source by ID, verifies it belongs to the
  requested Media Library, derives only source scheme from the current Source
  Locator, and persists `SourceFingerprintHashJobInput`;
- persisted jobs use the `disk.scan.source_fingerprint_hash` resource class,
  `library_id`, `source_id`, optional priority, and redaction-safe input JSON.

Boundaries:

- no durable scheduler loop, lease executor, VFS read, Admin/Public API,
  schema migration, evidence persistence, duplicate relationship mutation, or
  automatic Media Source merge behavior was added.
- Missing sources, cross-library requests, and invalid source locators reject
  before enqueueing without echoing raw locator/path/fingerprint values.

### source-fingerprint-hash-queued-execution-planner-first-slice

Status: Internal queued execution planner seam shipped as of 2026-06-05.

Shipped:

- `nako-server::app::source_hash` can prepare a persisted
  `JobKind::SourceFingerprintHash` job for future execution by validating the
  job kind, resource class, redaction-safe input, and job/source bindings;
- the planner reloads the current Media Source by ID, verifies library
  ownership, parses the current Source Locator only into an in-memory
  `SourceFingerprintHashRequest`, and confirms the locator scheme still matches
  the persisted source scheme;
- malformed input, wrong job contracts, mismatched bindings, invalid locators,
  and scheme drift fail with messages that do not echo raw locator, path,
  query, fingerprint, or input JSON content.

Boundaries:

- no durable scheduler loop, lease executor, runtime worker, VFS read,
  Admin/Public API, schema migration, evidence persistence, duplicate
  relationship mutation, or automatic Media Source merge behavior was added.

### source-fingerprint-hash-job-summary-contract-first-slice

Status: Durable job summary contract shipped as of 2026-06-05.

Shipped:

- `nako-library::source_hash` exposes
  `SourceFingerprintHashJobSummary` for future durable executor summary JSON;
- the summary projects a `SourceFingerprintHashReport` into hash mode,
  evidence kind, confidence, stale state, and bytes hashed;
- summary serialization intentionally omits `SourceFingerprintEvidence` raw
  fingerprint material, raw digests, Source Locators, `StorageUri`, paths,
  backend URLs, etags, credentials, and job input JSON.

Boundaries:

- no durable scheduler loop, lease executor, runtime worker, VFS behavior
  change, Admin/Public API, schema migration, evidence persistence, duplicate
  relationship mutation, or automatic Media Source merge behavior was added.

### source-fingerprint-hash-durable-executor-command-first-slice

Status: Internal single-job durable executor command shipped as of 2026-06-05.

Shipped:

- `nako-server::app::source_hash` can execute one explicit
  `JobKind::SourceFingerprintHash` job id through `DurableJobRuntime`;
- the command claims the durable job, reuses the queued execution planner,
  resolves the configured VFS backend for the current Media Source, runs
  `SourceFingerprintHashExecutor`, and persists
  `SourceFingerprintHashJobSummary` in `summary_json`;
- focused tests prove successful execution marks the job succeeded, makes it no
  longer claimable, and keeps summary JSON free of Source Locators, `StorageUri`,
  raw digests, fingerprints, and hash material.

Boundaries:

- no automatic scheduler loop, startup worker, runtime-supervisor background
  spawn, Admin/Public API, schema migration, evidence persistence outside job
  summary JSON, duplicate relationship mutation, or automatic Media Source merge
  behavior was added.

### source-fingerprint-hash-scheduler-integration-first-slice

Status: Internal disk-scan scheduler integration shipped as of 2026-06-05.

Shipped:

- the existing `nako-server::app::jobs` disk-scan scheduler can consider
  queued `JobKind::SourceFingerprintHash` jobs alongside library scans;
- source hash scheduler candidates are queried through disk-scan kind/resource
  windows rather than an all-job preview window, then merged with durable queue
  priority/FIFO/starvation semantics;
- scheduler-originated execution claims the source hash job once, runs it
  through `SourceFingerprintHashAppService::execute_claimed_source_fingerprint_hash_job`,
  keeps the `disk.scan` permit alive, and persists redaction-safe summary or
  failure state;
- focused tests cover successful execution, unrelated claimable jobs filling the
  generic preview window, cross-kind starvation ordering, and redacted execution
  failures.

Boundaries:

- no source-hash-specific runtime loop, Admin/Public API, schema migration,
  evidence persistence outside job summary/error, duplicate relationship
  mutation, or automatic Media Source merge behavior was added.

### source-fingerprint-hash-evidence-persistence-first-slice

Status: Source fingerprint evidence persistence shipped as of 2026-06-05.

Shipped:

- `nako-server::app::source_hash` persists the redacted
  `SourceFingerprintHashReport` evidence fingerprint back onto the current
  `MediaSource`;
- when a matching `SourceState` exists for the current library and locator, the
  same redacted fingerprint is written back while preserving the other state
  facts;
- successful source fingerprint hash execution now leaves durable source
  identity evidence in the existing source records, without adding a separate
  evidence table or automatic duplicate reconciliation policy.

Boundaries:

- no Admin/Public API, schema migration, duplicate relationship mutation, or
  automatic Media Source merge behavior was added.

### vfs-cache-repair-diagnostics

Status: Minimal diagnostic slice shipped as of 2026-06-02; structured action
preview, latest-failure refresh, latest action plan, target-scoped previews, and
selected-target refresh execution shipped as of 2026-06-05.

Shipped:

- VFS cache repair diagnostics classify fresh cache, stale fallback repair,
  retryable refresh failures, operator-action failures, and unknown failures;
- diagnostics are derived from existing redaction-safe storage failure classes
  and never include source locators, raw provider errors, etags, fingerprints,
  or local paths;
- Admin repair previews now include a stable `recommended_action` enum for UI
  and operator routing while preserving display-oriented `operator_action`
  prose;
- Admin refresh is executable only through the latest unresolved
  `refresh_cache` route and is guarded by stored failure authority to avoid
  ambiguous backend targeting;
- Admin action plans classify latest repair diagnostics into no-action,
  API-executable, and plan-only states, with route-key/path guidance only for
  the existing refresh route;
- Admin target inventory and preview routes expose bounded unresolved repair
  targets through process-keyed opaque `target_ref` values, safe
  scheme/operation/time/failure scope, and read-only action-plan previews
  without raw URI, local path, backend URL, etag, fingerprint, credential, or
  raw backend error body;
- target-scoped preview is intentionally non-mutating while refreshable targets
  can point to the selected-target refresh route;
- selected-target refresh resolves opaque `target_ref` values server-side,
  refreshes only unresolved diagnostics that recommend `refresh_cache`, and
  reuses stored failure authority so ambiguous or mismatched backend targeting
  fails before a backend call;
- purge/delete/invalidation, durable jobs, backend configuration mutation,
  library file writes, and retry queues remain out of this shipped boundary;
- no storage schema, playback artifact pressure, or scan scheduling expansion
  was added; Admin API changes stayed limited to redaction-safe diagnostics,
  action planning, latest-failure refresh, target previews, and selected-target
  refresh.

## Next Work Lanes

- `proposed:vfs-cache-repair-non-destructive-remediation`: stale-cache operator
  remediation planning, durable repair queues, and broader non-destructive
  repair guidance beyond refresh-only target actions.
- `.trellis/tasks/archive/2026-06/06-02-01d-hls-artifact-io-pressure-enforcement/`
  (closed HLS artifact I/O pressure admission; shipped by `48668afc`).
- `proposed:source-fingerprint-hash-queue-and-operator-integration`: durable
  scan/operator scheduling, lease execution, Admin diagnostics/API triggering,
  and controlled execution around the shipped advisory planner, durable job
  contract, internal enqueue seam, partial/full hash execution kernel, and
  scheduler integration. The current scan escalation policy seam remains
  advisory and does not read source bytes.
- `proposed:storage-vfs-postgresql-runtime-harness`: runtime parity evidence
  for PostgreSQL storage/source identity query paths.

## Risk Register

### OS Mounts Can Block Like Local Files

SMB/NFS/rclone mounts often look like local paths but behave like remote
services. Treating every mounted path as safe local disk can stall scan, probe,
or playback.

Mitigation:

- isolate blocking local filesystem calls behind bounded permits;
- use timeout wrappers around probe/stage workflows;
- do not hold global locks while touching mounted paths.

### Fingerprint Policy Can Be Too Expensive

Hashing entire multi-gigabyte files during scan can hurt NAS and cloud-backed
libraries.

Mitigation:

- prefer layered evidence: size, mtime, path, duration, stream facts, partial
  hash, then full hash only when needed;
- record confidence and escalation recommendations instead of forcing exact
  identity for every source.

### Remote Staging Can Leak Disk

Interrupted probe or playback staging can leave large temporary files.

Mitigation:

- keep staging manifests authoritative;
- run startup cleanup;
- record ownership by library/source/session;
- expose Admin diagnostics for stale staging.

## Agent Notes

Before changing scan, probe, playback input staging, or sidecar write behavior,
read ADR 0016 and ADR 0017. Do not bypass VFS with raw `std::fs` in application
logic unless the module is explicitly a local-backend adapter.
