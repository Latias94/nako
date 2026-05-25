# Metadata Acquisition Pipeline Evidence And Gates

## Gates

Run focused gates first:

```powershell
cargo nextest run -p nako-core metadata_profile --no-fail-fast
cargo nextest run -p nako-server scan_library_enqueues_addon_bulk_metadata_scrape_when_enabled --no-fail-fast
cargo nextest run -p nako-api openapi sdk --no-fail-fast
cargo fmt --all -- --check
```

Broaden when MAP-040 lands:

```powershell
cargo nextest run -p nako-server scan_library addon_bulk_metadata --no-fail-fast
cargo nextest run -p nako-core -p nako-api -p nako-server --no-fail-fast
```

## Evidence

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-25 | MAP-010 | Workstream docs created for metadata acquisition pipeline, explicit Addon writeback policy, and closed-loop side-effect proof. | Pass |
| 2026-05-25 | MAP-020 | Extracted scan-time metadata acquisition into `crates/nako-server/src/app/metadata_scan.rs`; verified `cargo nextest run -p nako-server scan_library_imports_enabled_nfo_metadata_after_probe --no-fail-fast`, `cargo nextest run -p nako-server scan_library_skips_nfo_import_when_scan_metadata_is_disabled --no-fail-fast`, and `cargo nextest run -p nako-server scan_library_enqueues_addon_bulk_metadata_scrape_when_enabled --no-fail-fast`. | Pass |
| 2026-05-25 | MAP-030 | Added `scan.addon_writeback` in core profile, public DTO/OpenAPI/SDKs, and official Addon `writeback` payload generation; verified `cargo nextest run -p nako-core metadata_profile_builds_scan_acquisition_plan_from_local_readers_and_policy --no-fail-fast`, `cargo nextest run -p nako-api openapi sdk --no-fail-fast`, and `cargo nextest run -p nako-server scan_library_adds_addon_bulk_metadata_writeback_when_enabled --no-fail-fast`. | Pass |
| 2026-05-25 | MAP-040 | Added in-process Nako HTTP + test sidecar loop where bulk scrape submits `/addon/v1/side-effects` and Media Item metadata is merged; verified `cargo nextest run -p nako-server scan_library_addon_bulk_metadata_writeback_merges_metadata_via_side_effect --no-fail-fast`. | Pass |
| 2026-05-25 | MAP-020/MAP-030/MAP-040 | `cargo nextest run -p nako-server scan_library --no-fail-fast` | Pass: 7 passed, 294 skipped |
| 2026-05-25 | MAP-050 local real-directory smoke | `pwsh -NoProfile -ExecutionPolicy Bypass -File target\codex-smoke\metadata-acquisition-smoke.ps1`; local run directory `target\codex-smoke\metadata-acquisition-local-20260525-100136`; library root `H:\Super\Videos`. | Pass: discovered 5 files, inserted 5 sources, probed 5, failed probes 0, discovered/imported 3 NFO files, health `ok`, playback decision `direct_play`, Range stream 206 with 1024 bytes. |
| 2026-05-25 | MAP-050 NAS real-directory smoke | Same smoke script; NAS run directory `target\codex-smoke\metadata-acquisition-nas-20260525-100138`; library root `\\frankorz-nas\home\www\Data\Video\Super\JAV_output\安位カヲル`. | Pass: discovered 5 files, inserted 5 sources, probed 5, failed probes 0, discovered/imported 5 NFO files, health `ok`, playback decision `direct_play`, Range stream 206 with 1024 bytes. |
| 2026-05-25 | MAP-060 closeout | `python -m json.tool docs\workstreams\metadata-acquisition-pipeline\WORKSTREAM.json`; `git diff --check -- docs\workstreams\metadata-acquisition-pipeline docs\workstreams\README.md`. | Pass |
| 2026-05-25 | Formatting | `cargo fmt --all -- --check` was attempted but failed on pre-existing unrelated dirty files in addon event/db tests. To avoid formatting user/parallel work, only the files touched by this lane were formatted with `rustfmt --edition 2024 ...`. | Partial |
