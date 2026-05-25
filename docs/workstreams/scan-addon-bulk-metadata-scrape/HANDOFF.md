# Scan Addon Bulk Metadata Scrape — Handoff

Status: Complete
Last updated: 2026-05-25

## Current State

The lane implemented scan-time automatic creation of bounded Addon `bulk-metadata-scrape` TaskRuns.

Shipped behavior:

- `MetadataScanPolicy` now has explicit `addon_scrape`, defaulting to false.
- Public DTO/OpenAPI/TypeScript/Kotlin SDK surfaces include `addon_scrape`.
- `LibraryScanAppService` calls `AddonAppService` when the scan acquisition plan enables Addon scrape.
- `AddonAppService` creates direct-dispatched, bounded official-compatible `bulk-metadata-scrape` TaskRuns for enabled Addons with executable task routing plans.
- Scan payloads include source/item query facts and omit implicit `writeback` / `artwork_writeback`.

## Verification

Passed on 2026-05-25:

- `cargo nextest run -p nako-core metadata_profile_builds_scan_acquisition_plan_from_local_readers_and_policy`
- `cargo nextest run -p nako-server config_applies_library_metadata_profile_overrides config_applies_library_metadata_addon_scrape_policy scan_library_enqueues_addon_bulk_metadata_scrape_when_enabled`
- `cargo nextest run -p nako-api openapi sdk`
- `cargo fmt --all -- --check`

## Follow-Ons

- Cursor continuation from `next_cursor`.
- UI/admin configuration for metadata acquisition source ordering.
- Event scheduler/replay remains in `docs/workstreams/addon-event-scheduler-and-replay/`.
