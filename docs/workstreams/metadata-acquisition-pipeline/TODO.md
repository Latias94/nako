# Metadata Acquisition Pipeline TODO

- [x] MAP-010 [owner=planner] [deps=none] [scope=docs/workstreams/metadata-acquisition-pipeline]
  Goal: Open the workstream and state the scan-time metadata acquisition
  boundary.
  Validation: Workstream docs exist and the lane is linked from the workstream
  index.
  Result: DONE 2026-05-25. Active lane created with design, milestones,
  evidence gates, and handoff notes.

- [x] MAP-020 [owner=worker] [deps=MAP-010] [scope=crates/nako-server/src/app]
  Goal: Extract scan-time metadata acquisition from `LibraryScanAppService`
  into a focused service while preserving current NFO and Addon behavior.
  Validation: Existing library scan, NFO import, cancellation, and Addon bulk
  scrape tests pass.
  Result: DONE 2026-05-25. Added `metadata_scan` application service,
  preserved scan output shape, and kept NFO/Add-on behavior behind the same
  scan acquisition plan.

- [x] MAP-030 [owner=worker] [deps=MAP-020] [scope=crates/nako-core,crates/nako-api,crates/nako-client-protocol,sdk,crates/nako-server/src/app/addons]
  Goal: Add an explicit scan policy for Addon metadata writeback and include the
  official Addon `writeback` payload only when enabled.
  Validation: Core plan tests cover defaults and disabled scan behavior; API,
  OpenAPI, TypeScript SDK, Kotlin SDK, and server payload tests cover the new
  field.
  Result: DONE 2026-05-25. Added disabled-by-default `addon_writeback` policy,
  public DTO/OpenAPI/SDK exposure, and official bulk scrape `writeback` payload
  generation for Media Source targets.

- [x] MAP-040 [owner=worker] [deps=MAP-030] [scope=crates/nako-server/src/app/tests,crates/nako-server/src/http]
  Goal: Prove the closed loop where scan-triggered Addon bulk scrape submits a
  metadata_write side effect and Canonical Metadata is merged for the scanned
  item.
  Validation: In-process HTTP test starts Nako and a test Addon sidecar, waits
  for the Addon TaskRun to succeed, and asserts the Media Item metadata source.
  Result: DONE 2026-05-25. Added a sidecar/Nako HTTP loop test where
  scan-triggered bulk scrape submits `metadata_write` via `/addon/v1/side-effects`
  and the scanned Media Item receives the merged Canonical Metadata.

- [x] MAP-050 [owner=worker] [deps=MAP-040] [scope=scripts,target/codex-smoke]
  Goal: Re-run local directory scan/playback smoke and prepare for real NAS
  smoke with explicit Addon writeback enabled when an official sidecar is
  available.
  Validation: Record exact commands, directories, task status, scan summary, and
  playback byte-range result in evidence.
  Result: DONE 2026-05-25. Re-ran post-refactor scan/list/serve/playback smoke
  on `H:\Super\Videos` and the NAS SMB `安位カヲル` subdirectory. Both returned
  healthy HTTP service, `direct_play` playback decision, and 206 Range streaming.

- [x] MAP-060 [owner=planner] [deps=MAP-020,MAP-030,MAP-040,MAP-050] [scope=docs/workstreams/metadata-acquisition-pipeline]
  Goal: Close or split the lane after implementation evidence is fresh.
  Validation: Evidence table is current, TODO reflects completed work, and
  HANDOFF states remaining follow-ons.
  Result: DONE 2026-05-25. Lane closed with implementation, tests, formatting
  note, and real-directory smoke evidence. Full NAS-root scan and official
  sidecar process smoke are deferred follow-ons, not blockers for this lane.
