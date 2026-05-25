# 2026-05-25 — Implementation

Implemented SABMS-020 through SABMS-040.

Changes:

- Added `MetadataScanPolicy.addon_scrape` with default-off behavior.
- Exposed `addon_scrape` through public DTO, OpenAPI, TypeScript SDK, and Kotlin SDK.
- Added `addons/scan_metadata.rs` as the scan-time Addon metadata scrape orchestration module.
- Wired `LibraryScanAppService` to call `AddonAppService` when `scan_acquisition_plan().addon_scrape` is true.
- Added an app-level scan test proving a real scan creates and dispatches a bounded official-compatible `bulk-metadata-scrape` TaskRun.

Verification:

- `cargo nextest run -p nako-core metadata_profile_builds_scan_acquisition_plan_from_local_readers_and_policy`
- `cargo nextest run -p nako-server config_applies_library_metadata_profile_overrides config_applies_library_metadata_addon_scrape_policy scan_library_enqueues_addon_bulk_metadata_scrape_when_enabled`
- `cargo nextest run -p nako-api openapi sdk`
- `cargo fmt --all -- --check`
