# Playback Streaming Milestones

## M7.0: Playback Streaming Design Baseline

Outcome: Taru has a documented playback-streaming workstream, ADR, milestone
split, and remote playback hardening plan before runtime changes begin.

Status: completed.

Deliverables:

- ADR 0017 for playback streaming and remote hardening boundaries.
- Dedicated `playback-streaming` workstream.
- M7 milestone split and validation strategy.
- Remote direct body streaming, staging cleanup, error mapping, resource
  budget, and multi-library config design notes.
- Roadmap, goal map, ADR index, and workstream index updates.

Exit criteria:

- M7.1 is the next concrete implementation slice.
- M6 deferred playback hardening tasks have a new owner.
- Docs-only validation passes.

## M7.1: Remote Direct Body Streaming

Outcome: Remote direct play streams selected ranges through HTTP response
bodies without materializing the whole selected range in memory.

Status: completed.

Deliverables:

- VFS/app body abstraction for local files and remote range streams.
- Remote direct-play route using bounded streaming bodies.
- HEAD and Range behavior preserved for local and remote sources.
- WebDAV stream timeout through backend request policy.
- Resource-budget acquisition remains M7.4.
- Tests proving large remote ranges are not buffered as `Vec<u8>` bodies.

Exit criteria:

- Direct play can serve remote range-readable sources through streaming body
  responses.
- Local direct play behavior remains unchanged.
- Backpressure and cancellation use the HTTP body stream/drop boundary.
- Dedicated remote stream resource budgeting is added in M7.4.

## M7.2: Staging Manifest, Disk Budget, and Cleanup

Outcome: Remote staging has persistent records, configured disk limits, and a
cleanup path that can be audited and tested.

Status: completed.

Deliverables:

- Staging manifest domain model and repository.
- Disk budget configuration and enforcement.
- Startup cleanup and bounded background cleanup worker.
- Reuse validation by size, fingerprint, etag, and purpose.
- Tests for budget exhaustion, stale cleanup, reuse, and validation mismatch.

Exit criteria:

- Staged remote inputs cannot grow unbounded.
- Cleanup never removes active staged inputs.
- Staging failures map to stable playback errors.
- Runtime staging paths are connected to the manifest and budget policy.

## M7.3: Playback Error Taxonomy and HTTP Mapping

Outcome: Playback APIs expose stable failure categories for local, remote, and
transcode playback paths.

Status: completed for M7 stable HTTP mappings.

Deliverables:

- Typed playback/storage error taxonomy.
- HTTP mapping for remote not found, unauthorized, timeout, transient failure,
  stale cache fallback, unsupported range, budget exhaustion, validation
  mismatch, and FFmpeg failures.
- API docs and route tests for representative failure responses.

Exit criteria:

- Playback route handlers no longer collapse remote storage failures into
  generic internal errors.
- Error responses avoid credentials and raw backend internals.

## M7.4: Remote Playback Resource Budgets

Outcome: Remote playback work is bounded independently from listing/stat cache
work and FFmpeg CPU/GPU work.

Status: completed.

Deliverables:

- `playback.remote.stream` budget.
- `playback.remote.stage` budget.
- Optional cleanup budget if cleanup becomes concurrently expensive.
- Server config defaults and validation.
- Tests for direct streaming and staging concurrency limits.

Exit criteria:

- Remote direct streaming cannot starve scan/list/stat work.
- Remote staging cannot overrun transcode CPU/GPU budget decisions.

## M7.5: Multi-Library and Multi-Remote Backend Config

Outcome: Taru can configure multiple named libraries and remote backend
instances without relying on a single `[library.webdav]` preview overlay.

Status: completed.

Deliverables:

- Explicit library configuration model.
- Backend-specific WebDAV config blocks with secret references.
- Stable source URI root and library identity rules.
- Replacement of the current single-library preview config.
- Tests for multiple libraries, mixed local/WebDAV backends, and secret
  omission.

Exit criteria:

- Server startup can build more than one library backend.
- Scan/probe/playback can resolve sources to the correct configured backend.
- Single-library local setup remains supported through one `[[libraries]]`
  entry.

Foundation notes:

- `TaruServerConfig` uses `[[libraries]]` as the only server library
  configuration model.
- Startup upserts every configured library instead of only the first one.
- Playback, FFmpeg staging, scan/probe, and NFO paths resolve the storage
  backend from the source's configured library when the source is persisted.
- URI-root fallback is used only for unpersisted sources and rejects ambiguous
  matches.

## M7.6: Playback Streaming Stabilization

Outcome: M7 is documented, validated, and ready for broader remote playback
testing.

Status: completed after validation.

Deliverables:

- HTTP API and local setup docs for remote streaming, staging budgets, cleanup,
  and multi-library configuration.
- Test strategy updates and validation matrix.
- Known limitations and follow-up recommendations.
- Full workspace validation.

Exit criteria:

- Workspace validation gates pass.
- M7 known limitations are explicit.
- The next implementation goal is explicit.
