# NFO Sidecar Cancellation Checkpoints Design

Status: Complete
Last updated: 2026-05-19

## Problem

NFO jobs are durable and now run through `DurableJobRuntime`, but the current
NFO cancellation boundary is too coarse:

- `crates/taru-server/src/app/nfo.rs` checks cancellation before and after the
  whole `NfoService` call.
- `crates/taru-nfo/src/import.rs::import_library` loops over all media sources
  and can read/parse/commit many sidecars after a cancel request.
- `crates/taru-nfo/src/export.rs::export_library` loops over all media sources
  and can stat/read/render/write many sidecars after a cancel request.

That means Admin can request cancellation and the app can eventually
acknowledge it, but NFO work may keep doing source-level side effects until the
whole library operation returns.

## Target State

When this lane closes:

- `taru-nfo` exposes a sidecar checkpoint API that is independent of
  `taru-server`.
- Library import/export check before each source sidecar unit.
- Cancellation is represented as a distinct service outcome, not as an NFO
  failure and not as a generic `TaruError`.
- Existing `import_library` and `export_library` remain no-op checkpoint
  wrappers for callers that do not need cancellation.
- `taru-server` maps `DurableJobContext::check_cancelled()` into the NFO
  checkpoint API and maps an NFO cancelled outcome back to
  `DurableJobOperationError::Cancelled`.
- Cancelled NFO jobs do not write `NfoImported` or `NfoExported` outbox events.
- Docs state that cancellation stops before the next sidecar boundary, not in
  the middle of an already-started storage operation.

## In Scope

- `taru-nfo` checkpoint types and library-wide import/export variants.
- Import loop checkpoint before each `import_source` sidecar read/commit unit.
- Export loop checkpoint before each `export_source` sidecar stat/read/write
  unit.
- Server integration with `DurableJobContext`.
- Focused tests for service-level cancellation and server durable job
  acknowledgement.
- Redaction docs for Admin cancellation semantics.

## Out Of Scope

- Retry/backoff after cancellation or failure.
- Lease stealing, requeue, distributed scheduling, or worker balancing.
- Killing storage reads/writes after they have started.
- Transcode/ffprobe child-process cancellation.
- Reworking NFO XML preservation or backup retention.
- Public Client API changes.

## Architecture Direction

Do not make `taru-nfo` depend on `taru-server`. The NFO crate should define a
small domain-level checkpoint contract:

1. A checkpoint receives redacted sidecar identity such as operation kind,
   `library_id`, `source_id`, and `item_id`.
2. The checkpoint returns either `Continue` or `Cancel`.
3. Library-level service methods return either `Completed(summary)` or
   `Cancelled(partial_summary)`.
4. Existing methods call the new variants with a no-op checkpoint and unwrap the
   completed outcome.
5. Server code converts `Cancelled` into `DurableJobOperationError::Cancelled`
   so the durable runtime persists terminal `cancelled`.

The checkpoint payload must not carry `source_locator`, `nfo_uri`, XML, storage
handles, local paths, backups, or raw error strings.

## Checkpoint Placement

Import:

- after source listing and before each call to `import_source`;
- not inside `read_to_string`, XML parsing, metadata merge, or commit once that
  source unit has started.

Export:

- after source listing and before each call to `export_source`;
- not inside stat/read/render/write/backup once that source unit has started.

Single-source addon export already operates on one sidecar. This lane may add a
pre-source checkpoint-capable helper if useful, but it must not change Addon
Library File Write behavior unless tests prove compatibility.

## Redaction Policy

Allowed in checkpoint/debug/test assertions:

- job ID;
- library ID;
- media source ID;
- media item ID;
- operation kind.

Not allowed in Admin/Public DTOs or checkpoint payloads:

- Source Locator;
- sidecar `StorageUri`;
- local filesystem paths;
- XML payloads;
- backup URIs;
- raw storage handles;
- secrets, headers, tokens, or environment values.

## Closeout Condition

This lane can close when:

- `taru-nfo` import/export can stop before the next sidecar unit;
- server NFO jobs acknowledge that service-level cancellation as terminal
  `cancelled`;
- success outbox events are skipped for cancelled NFO jobs;
- focused tests prove import and export behavior;
- docs and workstream evidence record the redaction and non-goal boundaries.

## Closeout Result

The lane is closed. `taru-nfo` owns the redacted checkpoint contract and
library import/export stop before the next source sidecar unit when the
checkpoint returns cancel. `taru-server` maps durable job cancellation through
that contract for both import and export background jobs, persists terminal
`cancelled`, and skips `NfoImported`/`NfoExported` outbox publication.

No retry/backoff, lease stealing, child-process cancellation, Public Client API
shape, XML preservation, backup retention, or storage write policy changes were
added in this lane.
