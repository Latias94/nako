# Metadata Application Policy Seam - Evidence And Gates

Status: Completed
Last updated: 2026-05-26

## Smallest Current Repro

```powershell
cargo nextest run -p nako-server -E 'test(addon_side_effect_metadata_write)' --no-fail-fast
```

This proves the primary Addon metadata writeback apply path.

## Gate Set

### Documentation Gate

```powershell
python -m json.tool docs/workstreams/metadata-application-policy-seam/WORKSTREAM.json
git diff --check -- docs/workstreams/metadata-application-policy-seam docs/workstreams/README.md
```

### Addon Metadata Application Gate

```powershell
cargo nextest run -p nako-server -E 'test(addon_side_effect_metadata_write) | test(scan_library_addon_bulk_metadata_writeback_merges_metadata_via_side_effect)' --no-fail-fast
```

Proves Addon writeback policy behavior, catalog projection, and scan-triggered
writeback.

### Core Metadata Gate

```powershell
cargo nextest run -p nako-core metadata --no-fail-fast
```

Proves existing pure metadata profile and merge behavior remains stable.

### Formatting Gate

```powershell
cargo fmt --all -- --check
git diff --check
```

## Evidence Anchors

- `crates/nako-server/src/app/addons/metadata_write.rs`
- `crates/nako-server/src/app/addons/scan_metadata.rs`
- `crates/nako-server/src/app/metadata_application.rs`
- `crates/nako-server/src/http/tests/addons.rs`
- `crates/nako-server/src/app/tests/startup.rs`
- `crates/nako-core/src/media/merge.rs`
- `crates/nako-metadata/src/strategy.rs`
- `crates/nako-metadata/src/confirmation.rs`

## Notes

- Fresh verification is required before marking tasks or the lane complete.
- PostgreSQL adapter behavior is not the primary risk in this lane unless the
  Addon persistence commit shape changes.

## Fresh Evidence

2026-05-26, MPS-020 red characterization:

- Added Addon writeback tests for library-profile MissingOnly behavior, user
  field locks, same-source Addon locks, safe report shape, and existing catalog
  projection behavior.
- Added scan-time Addon writeback coverage proving library-profile MissingOnly
  should preserve an existing title while filling missing overview.
- Initial server gate failed as expected before the implementation because the
  old Addon path hard-coded `MetadataRefreshMode::FullRefresh`.

2026-05-26, MPS-030/MPS-040 implementation:

- Added `crates/nako-server/src/app/metadata_application.rs`.
- `MetadataApplication` now resolves library-profile refresh mode, loads field
  locks, applies source-aware `MetadataMergePolicy`, plans catalog graph/search
  projection, and returns a redacted apply report.
- `crates/nako-server/src/app/addons/metadata_write.rs` now parses and
  validates Addon payloads, maps the patch into `CanonicalMetadata`, resolves
  the Side Effect target, and delegates to `MetadataApplication`.
- The hard-coded Addon `MetadataRefreshMode::FullRefresh` and direct
  `plan_item_catalog_projection` call were removed from the Addon Adapter.

2026-05-26, MPS-050 provider/hierarchy audit:

- `crates/nako-metadata/src/provider_attempt.rs` already uses
  `MetadataMergePolicy::from_locks_and_mode` with refresh profiles and returns
  a provider refresh commit through the `MetadataRefreshPort`.
- `crates/nako-metadata/src/confirmation.rs` already uses
  `MetadataMergePolicy::for_source_refresh_mode` and hydrates catalog through
  its repository port.
- Decision: do not force these paths through `nako-server::app` because that
  would introduce the wrong dependency direction. If reuse pressure grows,
  extract pure application-decision command/result types into `nako-core`; keep
  repository/catalog side effects in their existing app/port adapters.

2026-05-26, closeout verification:

- `cargo nextest run -p nako-server -E 'test(addon_side_effect_metadata_write) | test(scan_library_addon_bulk_metadata_writeback_merges_metadata_via_side_effect)' --no-fail-fast`
  exited 0 with 8 selected tests passed.
- `cargo nextest run -p nako-server addon_side_effect --no-fail-fast` exited
  0 with 14 selected tests passed.
- `cargo nextest run -p nako-core metadata --no-fail-fast` exited 0 with 1
  selected test passed.
- `cargo fmt --all -- --check` exited 0.
- `python -m json.tool docs/workstreams/metadata-application-policy-seam/WORKSTREAM.json`
  exited 0.
- `git diff --check` exited 0. Git reported Windows LF-to-CRLF working-copy
  warnings only; no whitespace errors were reported.

Closeout review:

- Workstream compliance has no blocking findings. The shipped backend slice
  matches the scoped target and leaves official Addon cleanup plus bulk
  continuation as separate follow-ons.
- Code-quality review has no blocking findings. The new Module owns host
  policy/projection/reporting, and Addon `metadata_write` no longer contains
  merge-policy or catalog-projection decisions.
