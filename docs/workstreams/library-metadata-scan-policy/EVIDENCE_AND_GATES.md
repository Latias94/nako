# Library Metadata Scan Policy - Evidence And Gates

Status: Closed
Last updated: 2026-05-25

## Planned Gates

- `cargo fmt --all -- --check`
- focused `cargo nextest run -p nako-core metadata_profile --no-fail-fast`
- focused `cargo nextest run -p nako-server library_scan nfo --no-fail-fast`
- `cargo check -p nako-api -p nako-server --tests`
- real local smoke against `H:\Super\Videos`
- real NAS single-directory smoke against one child of
  `\\frankorz-nas\home\www\Data\Video\Super\JAV_output`
- `git diff --check`

## Closeout Result

Closed on 2026-05-25. The first scan-time metadata acquisition slice is limited
to NFO Import and is verified by focused unit/API/server tests plus real local
and SMB playback smoke.

## Evidence Log

| Date | Task | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-25 | LMSP-010 | Workstream opened after real local/NAS smoke showed scan and playback pass but NFO requires a separate import command. | Pass. First slice is scan-time NFO import through a profile-derived metadata acquisition plan. |
| 2026-05-25 | Reference | Jellyfin docs reviewed for operator-facing lesson: local `.nfo` metadata near media files and per-library metadata/image fetcher choices. | Pass. Nako will borrow the configuration intuition, not Jellyfin Plugin Compatibility or internal APIs. |
| 2026-05-25 | LMSP-020 | `cargo nextest run -p nako-core metadata_profile_builds_scan_acquisition_plan --no-fail-fast` | Pass. Profile-derived scan acquisition plan enables local NFO import by default and disables it when local metadata, NFO readers, or scan metadata are disabled. |
| 2026-05-25 | LMSP-020 | `cargo nextest run -p nako-server config_applies_library_metadata_profile_overrides --no-fail-fast` | Pass. Server TOML can override a library Metadata Profile through `metadata.library_profiles` and the override feeds the scan acquisition plan. |
| 2026-05-25 | LMSP-030 | `cargo check -p nako-server --bin nako-server` | Pass with existing dead-code warnings. Scan output/job summary compiles with metadata acquisition summary. |
| 2026-05-25 | LMSP-030 | `cargo nextest run -p nako-server scan_library_imports_enabled_nfo_metadata_after_probe scan_library_skips_nfo_import_when_scan_metadata_is_disabled --no-fail-fast` | Pass. Scan imports enabled NFO metadata after probe and skips NFO import when scan metadata is disabled. |
| 2026-05-25 | LMSP-030 | `cargo nextest run -p nako-api openapi public_client --no-fail-fast` | Pass. Public DTO/OpenAPI include the new metadata scan policy shape and existing public client redaction tests remain green. |
| 2026-05-25 | LMSP-030 | `cargo nextest run -p nako-core -p nako-client-protocol -p nako-api --no-fail-fast` | Pass after refreshing generated SDKs. 78 tests passed. |
| 2026-05-25 | LMSP-030 | `cargo nextest run -p nako-server scan_library_imports_enabled_nfo_metadata_after_probe scan_library_skips_nfo_import_when_scan_metadata_is_disabled config_applies_library_metadata_profile_overrides --no-fail-fast` | Pass. 3 tests passed. |
| 2026-05-25 | LMSP-030 | `cargo build -p nako-server --bin nako-server` | Pass with existing dead-code warnings. Built the binary used for real smoke. |
| 2026-05-25 | LMSP-040 | Local smoke config under `target/nako-smoke/metadata-local`; `scan`, `list`, `serve`, playback decision, and Range stream against `H:\Super\Videos`. | Pass. Scan discovered 5 files, inserted 5 sources, probed 5, discovered 3 NFO files, imported 3, failed 0. HTTP health 200, playback mode `direct_play`, Range stream 206 with 16 bytes. |
| 2026-05-25 | LMSP-040 | NAS smoke config under `target/nako-smoke/metadata-nas-single`; single directory `\\frankorz-nas\home\www\Data\Video\Super\JAV_output\愛花あゆみ,月野かすみ\MTALL-098 愛花あゆみ,月野かすみ`; `scan`, `list`, `serve`, playback decision, and Range stream. | Pass. Scan discovered 1 file, inserted 1 source, probed 1, discovered/imported 1 NFO, failed 0. HTTP health 200, playback mode `direct_play`, Range stream 206 with 16 bytes. |
| 2026-05-25 | Closeout | `rustfmt --edition 2024 --check crates/nako-api/src/openapi.rs crates/nako-api/src/public_client.rs crates/nako-client-protocol/src/catalog.rs crates/nako-core/src/media.rs crates/nako-core/src/media/profile.rs crates/nako-server/src/app/jobs.rs crates/nako-server/src/app/tests/startup.rs crates/nako-server/src/config.rs` | Pass. Scoped formatting check for this lane's Rust files. |
| 2026-05-25 | Closeout | `cargo fmt --all -- --check` | Blocked by unrelated unstaged addon/catalog files: `crates/nako-official-addon-catalog/src/lib.rs` and `crates/nako-server/src/app/addons.rs`. These files are outside the NFO scan acquisition slice and were left untouched. |
| 2026-05-25 | Closeout | `python -m json.tool docs/workstreams/library-metadata-scan-policy/WORKSTREAM.json > $null` | Pass. Workstream JSON is valid. |
| 2026-05-25 | Closeout | `git diff --check -- <lane files and docs>` | Pass. No whitespace errors in this lane's modified files. |
