# Scan Addon Bulk Metadata Scrape — TODO

Status: Active
Last updated: 2026-05-25

## M0 — Scope And Evidence Freeze

- [x] SABMS-010 [owner=planner] [deps=none] [scope=docs/workstreams/scan-addon-bulk-metadata-scrape]
  Goal: Freeze problem, target state, non-goals, and evidence anchors.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json exist and agree.
  Evidence: `docs/workstreams/scan-addon-bulk-metadata-scrape/DESIGN.md`
  Handoff: Current implementation should avoid the dirty addon-event scheduler workstream.

## M1 — Policy And Contract Slice

- [x] SABMS-020 [owner=codex] [deps=SABMS-010] [scope=crates/nako-core,crates/nako-api,crates/nako-client-protocol,crates/nako-server/src/config.rs]
  Goal: Add explicit `scan.addon_scrape` policy with default-off behavior and public DTO/OpenAPI representation.
  Validation: `cargo nextest run -p nako-core metadata_profile_builds_scan_acquisition_plan_from_local_readers_and_policy`; `cargo nextest run -p nako-server config_applies_library_metadata_profile_overrides`; `cargo nextest run -p nako-api openapi`.
  Review: Confirm backward-compatible defaults and no provider-specific policy names.
  Evidence: `crates/nako-core/src/media/profile.rs`; `crates/nako-api/src/openapi.rs`
  Handoff: DONE. `scan.addon_scrape` is default-off, configurable, and included in public DTO/OpenAPI/SDK outputs.

## M2 — Scan-To-TaskRun Slice

- [x] SABMS-030 [owner=codex] [deps=SABMS-020] [scope=crates/nako-server/src/app/jobs.rs,crates/nako-server/src/app/addons/task_runtime.rs,crates/nako-server/src/app/composition.rs]
  Goal: When `addon_scrape` is enabled, scan metadata acquisition enqueues bounded `bulk-metadata-scrape` TaskRuns for enabled Addons with executable task routing plans.
  Validation: `cargo nextest run -p nako-server scan_library_enqueues_addon_bulk_metadata_scrape_when_enabled`.
  Review: Confirm scan code does not apply Addon task results or inject implicit writeback payloads.
  Evidence: `crates/nako-server/src/app/tests/startup.rs`
  Handoff: DONE. Scan creates direct-dispatched bounded bulk metadata scrape TaskRuns through `AddonAppService`; payload omits implicit writeback requests.

## M3 — Final Verification And Closeout

- [x] SABMS-040 [owner=codex] [deps=SABMS-030] [scope=docs/workstreams/scan-addon-bulk-metadata-scrape]
  Goal: Run focused verification, update evidence, and summarize deferred follow-ons.
  Validation: `cargo nextest run -p nako-core -p nako-api -p nako-server scan`; broaden if risk requires it.
  Review: Workstream docs and code diff should exclude unrelated addon-event scheduler files.
  Evidence: `docs/workstreams/scan-addon-bulk-metadata-scrape/EVIDENCE_AND_GATES.md`
  Handoff: DONE. Follow-ons remain event scheduler/replay and cursor continuation.
