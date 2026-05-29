# Durable Job Queue And Resource Classes Design

Status: Closed
Last updated: 2026-05-29

## Why This Lane Exists

ADR 0053 makes durable jobs, runtime supervision, tracing, remote access, addon
lifecycle, and API scale part of the application control plane. Nako already has
important foundations:

- durable job rows and persisted inputs;
- fenced ownership leases and heartbeats;
- cooperative cancellation checkpoints for real workers;
- startup recovery for unfinished work;
- `RuntimeSupervisor` task/job diagnostics;
- feature-local resource permits for scan, metadata, and webhook work.

Those parts are still too scattered for the next wave of media-server work.
`job.resource_class` exists, but `RuntimeSupervisor::spawn_job` only records the
string. Permit enforcement is owned by feature services through separate
`Arc<Semaphore>` fields. Retry/backoff, queue pressure, and priority therefore
cannot reason about resource budgets through one boundary.

## Target State

When this lane closes, durable background work has a shared control-plane
shape:

1. Runtime resources expose named resource classes with configured concurrency
   and safe occupancy diagnostics.
2. Durable job workers continue to be typed. The scheduler chooses work; typed
   executors still own domain side effects and transaction boundaries.
3. Retry/backoff rules are explicit and persisted enough to survive process
   restarts. Priority policy is split to a follow-on lane.
4. Cancellation and lease ownership remain fenced by `job_id + run_token`.
5. Queue pressure diagnostics describe counts and resource classes without raw
   inputs, provider payloads, local paths, source locators, or secrets.

## First Slice

The first slice does not need a new crate or an external dependency. A local
registry in `nako-server::app::runtime` is enough because all current runtime
permit pools are process-local and owned by server composition.

The first implementation should:

- introduce a `RuntimeResourceClassRegistry`;
- store `Arc<Semaphore>` pools behind named classes;
- expose deterministic diagnostics with class name, max permits, and available
  permits;
- build scan, metadata, and webhook pools from the registry in
  `NakoRuntimeResources`;
- keep existing service constructors stable by cloning the registry-owned
  semaphores;
- add focused tests for duplicate class rejection, diagnostics, and permit
  accounting.

This is deliberately not yet a scheduler. It removes the current resource
ownership scatter so the next scheduler slice has a real seam.

## In Scope

- Runtime resource class registry and diagnostics in `nako-server`.
- Process-local resource budget construction during app composition.
- Mapping existing scan, metadata, and webhook permit pools to named registry
  classes.
- Workstream/index updates under `docs/architecture` and `docs/workstreams`.
- Focused runtime tests and formatting/static gates.

## Out Of Scope

- Database schema changes.
- Distributed scheduling or multi-process balancing.
- External queues such as Redis, NATS, or SQLite queue tables in the first
  slice.
- Public Admin/Public Client wire contract changes in the first slice.
- Playback/HLS, transcode child-process cancellation, or remote worker
  execution.
- Addon manager process lifecycle.
- Full migration of every job kind in one PR-sized task.

## Crate Boundary Decision

Do not split a new crate for the first slice. The registry is currently useful
only to `nako-server` composition and runtime diagnostics. Moving it into a new
crate now would create an abstract control-plane crate before there are multiple
real production callers.

Reconsider a crate split only after at least two of these become true:

- the database repository needs shared queue/scheduling records outside server;
- addon/runtime workers need the same types without depending on `nako-server`;
- playback/transcode runtime scheduling needs a shared public resource policy
  model;
- client/admin protocol crates need stable DTOs for resource class diagnostics.

Until then, keep the implementation deep inside `nako-server` and move only
stable records or repository contracts into `nako-core`/`nako-db` when a later
slice proves the need.

## Resource Class Vocabulary

Initial process-local budget classes:

| Class | Budget Source | Current Users |
| --- | --- | --- |
| `addon.task` | `addon_event_scheduler.concurrency` | addon task dispatch and generated artifact handoff admission |
| `artwork.ingest` | `artwork.fetch_concurrency` | managed artwork ingest admission |
| `disk.scan` | `scan_concurrency` | library scan/probe workflow |
| `metadata.shared` | `metadata_concurrency` | provider refresh, metadata maintenance, NFO, addon metadata-side effects |
| `network.webhook` | `webhook_concurrency` | webhook delivery |

The names distinguish budget pools from the more specific durable
`job.resource_class` values such as `metadata.tmdb` or `metadata.nfo.export`.
`DJRC-030` added an explicit mapping from durable job classes to budget classes
before scheduler admission. New addon task jobs now use the fixed
`addon.task` class; the mapper accepts older `addon.task.*` and
`addon.generated_artifact_handoff` rows only as legacy compatibility.

## Scheduler Direction

Future scheduler work should claim queued jobs through typed job-kind handlers:

```text
queued durable jobs
  -> policy filter and priority
  -> resource class budget admission
  -> leased run token
  -> RuntimeSupervisor::spawn_job
  -> DurableJobRuntime typed executor
```

The scheduler must stay typed at the execution boundary. It may share admission
logic, retry policy, and diagnostics, but it should not become a dynamic JSON
function dispatcher.

## Diagnostics Policy

Safe diagnostics may expose:

- resource class name;
- configured max permits;
- currently available permits;
- active supervised task count;
- durable job counts by status;
- queue counts grouped by safe job kind or resource class after a later slice.

Diagnostics must not expose:

- raw `input_json`, `summary_json`, or `error`;
- provider payloads, addon payloads, NFO XML, or webhook bodies;
- local paths, source locators, storage handles, cache URIs, artifact roots, or
  content hashes;
- bearer tokens, sidecar tokens, worker run tokens, headers, environment values,
  or secrets.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| A local registry is enough for the first slice. | High | Existing scan, metadata, and webhook permits are process-local fields in server composition. | If a real shared caller appears, extract stable types after tests prove the boundary. |
| Existing typed worker constructors should remain stable initially. | High | Scan, metadata, NFO, addon, and webhook services already own side-effect semantics. | A broad migration would obscure the resource ownership change. |
| Budget class names should not be inferred from durable job class prefixes. | Medium | Current job classes are provider/job specific while permits are coarser pools. | Add explicit mapping in the scheduler slice. |
| Retry/backoff needs its own persisted policy slice. | High | ADR 0006 calls out explicit retry policy; prior lanes split it deliberately. | Mixing retry into the registry would couple unrelated responsibilities. |

## Closeout Condition

This lane closed when:

- process-local resource classes are centralized and visible in safe internal
  diagnostics;
- the scheduler can admit the first typed durable job family by budget without
  bypassing typed executors;
- retry/backoff behavior is explicit for at least network/provider work;
- queue pressure diagnostics are redacted and covered by tests;
- residual priority policy, distributed scheduling, remote worker, addon
  lifecycle, child-process cancellation, and broader job-kind migration work is
  split explicitly.
