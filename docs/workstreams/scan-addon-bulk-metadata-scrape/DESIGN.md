# Scan Addon Bulk Metadata Scrape

Status: Complete
Last updated: 2026-05-25

## Why This Lane Exists

Nako can scan a Media Library, import NFO metadata during scan, register Addons, synchronize Addon routing plans, and create Addon TaskRuns manually. The missing loop is scan-time Addon metadata acquisition: a library profile cannot explicitly opt into Addon scrape, and a completed scan does not enqueue the official or compatible `bulk-metadata-scrape` Addon Task.

## Relevant Authority

- Glossary: `CONTEXT.md`
- Existing workstreams:
  - `docs/workstreams/addon-ecosystem-foundation/`
  - `docs/workstreams/addon-event-scheduler-and-replay/`
- Existing code:
  - `crates/nako-core/src/media/profile.rs`
  - `crates/nako-server/src/app/jobs.rs`
  - `crates/nako-server/src/app/addons/task_runtime.rs`
  - `crates/nako-official-addon-catalog/src/lib.rs`

## Problem

`MetadataScanAcquisitionPlan` already has an `addon_scrape` slot, but it is always false. Scan completion records a `library.scanned` event and executes NFO import only. Addon task creation is available through Admin APIs, but no scan policy drives it.

This leaves the metadata acquisition model partially file-centric: NFO is wired into scanning, while Addon scrape remains a manual post-scan action.

## Target State

- A Media Library profile can explicitly enable scan-time Addon scrape with `scan.addon_scrape = true`.
- When scan metadata acquisition reaches the Addon scrape phase, Nako creates bounded `bulk-metadata-scrape` Addon TaskRuns for enabled Addons that declare that task and have an executable routing plan.
- The task payload is a bounded list of Media Source / Media Item query facts and does not include implicit metadata or artwork writeback requests.
- Scan summaries expose which Addon scrape TaskRuns were created or skipped.
- Addon metadata and artwork writes remain under Addon Side Effect permissions; scan code does not directly apply task output.

## In Scope

- Metadata scan policy and public DTO shape.
- Scan-time Addon scrape orchestration.
- Bounded TaskRun payload generation from current Media Sources and Media Items.
- Focused tests for policy parsing and scan-to-TaskRun behavior.
- Workstream evidence and handoff docs.

## Out Of Scope

- Automatic Addon Event scheduler/replay.
- Addon task continuation from `next_cursor`.
- Provider-specific metadata matching, writeback, or artwork acceptance policy.
- Installing or supervising Addon Sidecars.
- Schema migrations.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The official metadata scraper task ID is the compatibility hook for scan-time Addon scrape. | High | `nako_official_addon_catalog::metadata_scraper::BULK_METADATA_SCRAPE_TASK_ID` | Compatible Addons may need a separate declared role later. |
| Scan should not fail just because an eligible Addon TaskRun cannot be created. | Medium | Addon Sidecars live outside the core trust boundary. | Operators may need a strict mode later. |
| Scan-enqueued payload must not request writeback by default. | High | Addon Side Effect permission boundary owns metadata/artwork writes. | Writeback automation should be configured explicitly in a later lane. |
| One scan can enqueue bounded batches rather than a single unbounded task. | High | Official task input supports bounded `items` and `batch_size`. | Very large libraries may need scheduler backpressure and continuation policies later. |

## Architecture Direction

Keep Addon task validation and creation behind `AddonAppService`. `LibraryScanAppService` should depend on that service rather than duplicate manifest, grant, routing, idempotency, and dispatch rules.

The scan pipeline stays a coordinator:

1. Build `MetadataScanAcquisitionPlan` from the Media Library profile.
2. Execute local acquisition such as NFO import.
3. If `addon_scrape` is enabled, ask `AddonAppService` to enqueue scan-time `bulk-metadata-scrape` TaskRuns.
4. Record summaries only; do not consume Addon task results inside the scan job.

This keeps the metadata acquisition policy extensible without letting scanning become a provider-specific implementation point.

## Closeout Condition

This lane can close when:

- `scan.addon_scrape` is configurable and represented in public DTOs,
- scan-time Addon scrape creates bounded TaskRuns only for enabled executable Addons,
- focused tests pass,
- docs record the shipped behavior and deferred follow-ons,
- and unrelated addon-event scheduler changes remain untouched.
