# Metadata Merge Policy Unification TODO

Status: Completed
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

- [x] MMP-010 [owner=codex] [deps=none] [scope=docs/workstreams/metadata-merge-policy-unification]
  Goal: Open the workstream with problem, target state, non-goals, authority,
  gates, and first executable task.
  Validation: `git diff --check`.
  Evidence: `docs/workstreams/metadata-merge-policy-unification/DESIGN.md`.
  Handoff: Continue with MMP-020 before editing merge code.

## M1 - Current Behavior Characterization

- [x] MMP-020 [owner=codex] [deps=MMP-010] [scope=crates/taru-metadata,crates/taru-nfo]
  Goal: Add or identify characterization tests for provider full refresh,
  provider missing-only, NFO local-first, NFO remote-first, and cross-source
  field locks.
  Validation: `cargo nextest run -p taru-metadata merge --no-fail-fast`;
  `cargo nextest run -p taru-nfo nfo_service --no-fail-fast`.
  Review: `review-workstream` before accepting completion.
  Evidence: `crates/taru-metadata/src/merge.rs`,
  `crates/taru-nfo/src/import.rs`, `crates/taru-nfo/src/lib.rs`.
  Handoff: Added source-aware characterization tests for provider hierarchy
  confirmation and NFO import. Continue with MMP-030.

## M2 - Shared Policy Boundary

- [x] MMP-030 [owner=codex] [deps=MMP-020] [scope=crates/taru-core,crates/taru-metadata,crates/taru-nfo]
  Goal: Extract one shared Canonical Metadata merge authority boundary and make
  provider refresh, hierarchy confirmation, and NFO import use it.
  Validation: `cargo check -p taru-core --tests`; `cargo check -p
  taru-metadata --tests`; `cargo check -p taru-nfo --tests`; focused nextest
  commands from MMP-020.
  Review: `review-workstream` for crate boundary, behavior, and test coverage.
  Evidence: `crates/taru-core/src/media/merge.rs`,
  `crates/taru-nfo/src/import.rs`.
  Handoff: Shared policy now lives in `taru-core`; `taru-metadata` re-exports
  `MetadataMergePolicy`; NFO import uses `for_nfo_import`.

## M3 - Integration And Documentation

- [x] MMP-040 [owner=codex] [deps=MMP-030] [scope=docs,crates/taru-metadata,crates/taru-nfo]
  Goal: Update docs and evidence so ADR/workstream language matches the shipped
  source-aware merge policy.
  Validation: `cargo fmt --all -- --check`; targeted package nextest gates;
  `git diff --check`.
  Review: `review-workstream` before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, any updated ADR or workstream docs.
  Handoff: Workstream docs now describe the shipped `taru-core` source-aware
  merge boundary. Continue with MMP-050 closeout.

## M4 - Closeout

- [x] MMP-050 [owner=codex] [deps=MMP-040] [scope=docs/workstreams/metadata-merge-policy-unification]
  Goal: Close the lane or split a narrower follow-up.
  Validation: `verify-rust-workstream` records fresh final gate evidence.
  Review: no blocking review findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Lane closed. Provider priority configuration and richer merge
  diagnostics remain deferred follow-ons.
