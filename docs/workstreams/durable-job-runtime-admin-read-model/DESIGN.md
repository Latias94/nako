# Durable Job Runtime And Admin Read Model Design

Status: Completed
Last updated: 2026-05-17

## Problem

ADR 0019 says background work should be registered through an explicit runtime
supervisor and that feature services should not own detached task lifecycle
details. M38 and M41 moved Nako in that direction: background tasks go through
`RuntimeSupervisor`, and startup now recovers unfinished queued/running durable
jobs.

The remaining server-side friction is that durable job lifecycle persistence is
still duplicated across workflow services:

- library scan starts, succeeds, fails, and serializes job summaries in
  `crates/nako-server/src/app/jobs.rs`;
- metadata refresh and maintenance do the same in
  `crates/nako-server/src/app/metadata.rs`;
- NFO import/export do the same in `crates/nako-server/src/app/nfo.rs`;
- runtime diagnostics know active task handles, but not durable job list/filter
  semantics;
- Admin API v1 overview can show runtime counters, but the web console still
  lacks a real Jobs/Tasks read model.

The current Interface is shallow: callers must know too much about durable job
state transitions, summary serialization, error persistence, and which
resource class each workflow uses. Deleting the scattered job lifecycle code
would push the same complexity into every workflow again, which means a deeper
Module is warranted.

## Target State

Introduce a deeper server-side durable job runtime Module that owns common job
lifecycle behavior:

- creating or accepting an already-enqueued durable job;
- starting the job exactly once;
- running the workflow future;
- serializing typed summaries;
- persisting success or failure;
- reporting supervised job diagnostics in one vocabulary;
- preserving startup recovery semantics from M41.

Add the first Admin API v1 job read model:

- `GET /admin/v1/jobs`;
- filter by status, kind, resource class, Media Library, Media Source, and
  pagination when supported by the repository;
- return admin-owned redacted list DTOs from `nako-api::admin`;
- keep existing root-level `GET /jobs/{job_id}` for compatibility while Admin
  API migration continues.

## Scope

In scope:

- `crates/nako-server/src/app/runtime.rs`
- `crates/nako-server/src/app/jobs.rs`
- `crates/nako-server/src/app/metadata.rs`
- `crates/nako-server/src/app/nfo.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-api/src/admin.rs`
- `crates/nako-core/src/repository/jobs.rs`
- `crates/nako-db/src/jobs.rs`
- focused server/API/db tests
- admin-web-console and goal evidence updates

Out of scope:

- no frontend UI scaffold;
- no Public Client API, public OpenAPI, TypeScript SDK, Rust SDK, or
  `nako-client-protocol` changes;
- no generic distributed queue, retry policy, resumable execution, or worker
  process model;
- no new Addon Task execution semantics;
- no broad job cancellation unless it falls out of a narrow read-model need;
- no playback session list/filter in this slice.

## Architecture Direction

Keep `NakoApp` as a composition root. Add depth behind server application
services rather than widening HTTP handlers.

The durable job runtime Module should be server-owned and AGPL. It should not
move into `nako-core` unless multiple non-server adapters genuinely need the
same behavior. `nako-core` should own durable records, IDs, filters, and
repository traits only.

Admin job DTOs belong in `nako-api::admin`, following ADR 0027. Public client
protocol crates must remain untouched.

The Admin API should expose safe operational state, not raw internal task
handles or server implementation details. Resource class and job IDs are safe;
secrets, raw provider bodies, local filesystem paths, and transient task IDs
are not stable Admin API concepts unless explicitly documented.

M54 applies this by making the job list response a list summary rather than a
raw job detail dump. `AdminJobListItem` exposes identity, type, status,
resource class, scope IDs, timestamps, and `has_*` flags. It intentionally does
not expose raw `input_json`, `summary_json`, or `error`.

## Implementation Findings

- Durable summary serialization must be part of the lifecycle Module. If a
  workflow succeeds but summary serialization fails, the durable job must be
  marked failed instead of staying in a running state.
- Admin list query parsing should not rely on extractor-level enum or flattened
  pagination rejection. M54 parses job list filters inside the handler path so
  errors can use Nako's API error envelope.

## Related Architecture Findings

This workstream records these server-side deepening candidates discovered
after M52:

1. Durable job lifecycle is duplicated across scan, metadata, and NFO workflow
   services.
2. Admin overview currently has runtime counters but no job list/filter
   drill-down.
3. Runtime supervision and durable job state are adjacent but not yet one deep
   Module.
4. Admin route orchestration should move toward app-level read models before
   adding more `/admin/v1/*` routes.
5. Playback session diagnostics and storage runtime diagnostics remain follow-on
   seams, but job list/filter is the next most useful Admin Console data source.
