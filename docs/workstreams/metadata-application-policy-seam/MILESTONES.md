# Metadata Application Policy Seam - Milestones

Status: Completed
Last updated: 2026-05-26

## M0 - Scope Freeze

Exit criteria:

- Workstream documents exist.
- Scope is limited to host metadata application.
- Addon adapter cleanup and bulk continuation are explicit follow-ons.

Primary evidence:

- `DESIGN.md`
- `TODO.md`

## M1 - Characterization

Exit criteria:

- Addon writeback tests cover MissingOnly behavior.
- Field locks from other sources protect Addon writes.
- Same-source Addon refresh behavior is explicit.
- Addon-sourced catalog graph/search projection remains verified.
- Scan-time Addon writeback proves host policy selection.

Primary gates:

- `cargo nextest run -p nako-server -E 'test(addon_side_effect_metadata_write) | test(scan_library_addon_bulk_metadata_writeback_merges_metadata_via_side_effect)' --no-fail-fast`

## M2 - Deep Module

Exit criteria:

- `MetadataApplication` owns lock lookup, merge policy, projection, and apply
  report for Addon metadata application.
- Its Interface is smaller than the behavior it hides.
- It does not introduce a dependency cycle with `nako-metadata`.

Primary gates:

- M1 server gate
- `cargo nextest run -p nako-core metadata --no-fail-fast`

## M3 - Adapter Cleanup

Exit criteria:

- `metadata_write.rs` only parses/validates/maps payloads and delegates.
- Hard-coded Addon `FullRefresh` is removed.
- Scan-time Addon writeback uses host profile-derived application mode.

Primary gates:

- M1 server gate

## M4 - Closeout

Exit criteria:

- Provider refresh and hierarchy confirmation reuse opportunities are audited.
- Evidence is recorded.
- Follow-ons are split/deferred.
- Workstream status matches reality.

Primary gates:

- `cargo fmt --all -- --check`
- `git diff --check`
- `python -m json.tool docs/workstreams/metadata-application-policy-seam/WORKSTREAM.json`

Closeout status: completed on 2026-05-26. M0 through M4 are satisfied by the
new `MetadataApplication` Module, Addon Adapter cleanup, characterization
coverage, provider/hierarchy audit, and recorded verification gates.
