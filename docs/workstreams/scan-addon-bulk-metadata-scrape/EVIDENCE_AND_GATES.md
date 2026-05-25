# Scan Addon Bulk Metadata Scrape — Evidence And Gates

Status: Active
Last updated: 2026-05-25

## Gates

| Gate | Command | Status | Evidence |
| --- | --- | --- | --- |
| Policy unit | `cargo nextest run -p nako-core metadata_profile_builds_scan_acquisition_plan_from_local_readers_and_policy` | Passed | 2026-05-25 |
| Config parse | `cargo nextest run -p nako-server config_applies_library_metadata_profile_overrides config_applies_library_metadata_addon_scrape_policy` | Passed | 2026-05-25 |
| Public OpenAPI and SDK | `cargo nextest run -p nako-api openapi sdk` | Passed | 2026-05-25 |
| Scan-to-addon task | `cargo nextest run -p nako-server scan_library_enqueues_addon_bulk_metadata_scrape_when_enabled` | Passed | 2026-05-25 |
| Formatting | `cargo fmt --all -- --check` | Passed | 2026-05-25 |

## Evidence Notes

- 2026-05-25: Workstream opened after confirming `addon_scrape` was hard-coded false and scan only ran NFO import.
- 2026-05-25: Implemented `scan.addon_scrape`, scan-time bulk metadata scrape TaskRun creation, and refreshed public SDK outputs.
- 2026-05-25: Verified focused gates for core policy, server config, API OpenAPI/SDK, scan-to-Addon TaskRun, and workspace formatting.

## Deferred Follow-Ons

- Event scheduler/replay and forced replay belong to `docs/workstreams/addon-event-scheduler-and-replay/`.
- Addon task continuation from `next_cursor` needs a scheduler/backpressure policy before enabling full-library automatic continuation.
- User-facing UI for metadata acquisition source ordering can layer on top of `MetadataScanPolicy`.
