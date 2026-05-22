# Server Startup Workflow And Durable Job Runtime Deepening

Status: Completed
Last updated: 2026-05-17

## Why This Lane Exists

M24 made `NakoApp` much thinner, but startup side effects have continued to
accumulate inside `NakoApp::new_with_store`: migration execution, stale
transcode session recovery, staging cleanup, configured library upsert,
metadata raw-cache cleanup, and metadata lifecycle task registration.

The runtime supervisor also correctly centralizes `tokio::spawn`, but durable
job execution still leaks across app services as repeated
`spawn + finish_*_job` patterns. That makes each background workflow own its
own success/failure logging and runtime diagnostics shape.

## Target State

- `NakoApp` remains the composition root for app service handles.
- Startup side effects live behind a `ServerStartupWorkflow` module with a
  test-visible startup report.
- Runtime supervision exposes a durable job execution helper that accepts job
  identity, resource class, and a future returning a `Result`.
- The first job-runtime slice covers library scan, metadata refresh, and
  metadata maintenance background jobs.
- Existing behavior stays compatible.

## In Scope

- Add a `server-runtime-deepening` workstream and M38 goal docs.
- Add `nako-server::app::startup` or equivalent.
- Move startup sequencing out of `NakoApp::new_with_store` without changing
  behavior.
- Add runtime diagnostics for supervised durable jobs.
- Replace the first duplicated job `spawn` wrappers in library scan and
  metadata app services.
- Add focused tests for startup reports, startup side effects, runtime job
  diagnostics, and job state updates.

## Out Of Scope

- Playback source selection, transcode plan, client profile, subtitle/HDR/audio
  selection, bandwidth, endpoint, or HLS ladder design.
- NFO round-trip preservation, unknown XML field retention, partial XML update,
  conflict reports, or soft/hard-link policy.
- Broad repository trait splitting. Repository seam deepening is a follow-on.
- Migrating webhook, automation, addon, NFO, or playback runner execution in
  this first slice.
- Public HTTP API, SDK, CLI, or database schema changes.

## Architecture Direction

Use `ServerStartupWorkflow` as the startup interface. Callers should not need
to know each startup side effect or its ordering. The report is the test
surface.

Use `RuntimeSupervisor::spawn_job` as the first durable job runtime interface.
It should concentrate job naming, resource class, success/failure accounting,
and logging. The individual app service still owns the actual workflow and
persisted job state for this slice.

## Closeout Condition

M38 can close when:

- startup side effects are no longer implemented directly in
  `NakoApp::new_with_store`;
- startup reports are tested;
- at least library scan and metadata background jobs use the deeper runtime job
  helper;
- runtime diagnostics expose job success/failure counts;
- focused and workspace validation gates pass.
