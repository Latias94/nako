# Metadata Application Policy Seam - TODO

Status: Completed
Last updated: 2026-05-26

## M0 - Workstream Open

- [x] MPS-010 [owner=planner] [deps=none] [scope=docs/workstreams/metadata-application-policy-seam]
  Goal: Open the host metadata application seam lane and freeze scope.
  Validation: Workstream docs exist and `WORKSTREAM.json` is valid JSON.
  Evidence: `DESIGN.md`; `TODO.md`; `WORKSTREAM.json`.
  Handoff: First executable implementation task is MPS-020.

## M1 - Characterize Addon Application Policy

- [x] MPS-020 [owner=codex] [deps=MPS-010] [scope=crates/nako-server/src/http/tests/addons.rs,crates/nako-server/src/app/tests/startup.rs]
  Goal: Add behavior tests before refactoring Addon metadata writeback.
  Validation: `cargo nextest run -p nako-server -E 'test(addon_side_effect_metadata_write) | test(scan_library_addon_bulk_metadata_writeback_merges_metadata_via_side_effect)' --no-fail-fast`.
  Review: review-workstream for policy coverage before accepting completion.
  Evidence: Addon writeback tests prove MissingOnly, field locks, same-source overwrite, Addon catalog projection, safe apply report shape, and scan-time policy behavior.
  Handoff: Completed with the MPS-030/MPS-040 implementation.

## M2 - MetadataApplication Module

- [x] MPS-030 [owner=codex] [deps=MPS-020] [scope=crates/nako-server/src/app/metadata_application.rs,crates/nako-server/src/app.rs]
  Goal: Add server app `MetadataApplication` Module that owns locks, merge policy, catalog projection, and apply report.
  Validation: focused server tests from MPS-020 plus `cargo nextest run -p nako-core metadata --no-fail-fast`.
  Review: review-workstream for Module depth, dependency direction, and Interface size.
  Evidence: `crates/nako-server/src/app/metadata_application.rs`.
  Handoff: Repository/catalog side effects stayed in `nako-server`; only move pure types later if a second crate needs them.

## M3 - Addon Adapter Refactor And Scan Host Policy

- [x] MPS-040 [owner=codex] [deps=MPS-030] [scope=crates/nako-server/src/app/addons/metadata_write.rs,crates/nako-server/src/app/addons/scan_metadata.rs]
  Goal: Convert Addon `metadata_write` to a thin Adapter and route scan-time writeback through host application mode.
  Validation: `cargo nextest run -p nako-server -E 'test(addon_side_effect_metadata_write) | test(scan_library_addon_bulk_metadata_writeback_merges_metadata_via_side_effect)' --no-fail-fast`.
  Review: review-workstream for removed hard-coded `MetadataRefreshMode::FullRefresh`, Addon policy ignorance, and report redaction.
  Evidence: `metadata_write.rs` no longer calls `MetadataMergePolicy` or `plan_item_catalog_projection` directly; scan-time writeback proves library-profile `MissingOnly` behavior through the Side Effect runtime.
  Handoff: Do not change official Addon behavior beyond host policy semantics.

## M4 - Provider/Hierarchy Audit And Closeout

- [x] MPS-050 [owner=codex] [deps=MPS-040] [scope=crates/nako-metadata,docs/workstreams/metadata-application-policy-seam]
  Goal: Audit provider refresh and hierarchy confirmation against the new Module and document follow-ons without creating dependency cycles.
  Validation: `cargo nextest run -p nako-server -E 'test(addon_side_effect_metadata_write) | test(scan_library_addon_bulk_metadata_writeback_merges_metadata_via_side_effect)' --no-fail-fast`; `cargo nextest run -p nako-core metadata --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`; `python -m json.tool docs/workstreams/metadata-application-policy-seam/WORKSTREAM.json`.
  Review: review-workstream and verify-rust-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`; `HANDOFF.md`; `WORKSTREAM.json`.
  Handoff: Official Addon adapter cleanup and scan Addon bulk continuation remain follow-on workstreams.
