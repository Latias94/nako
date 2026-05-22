# nako-api Module Split Task Ledger

Status: Completed
Last updated: 2026-05-17

## M0 - Scope And Evidence Freeze

- [x] AMS-010 [owner=codex] [deps=none] [scope=docs/workstreams/api-module-split,docs/GOALS.md]
  Goal: Open M46 with module split scope, non-goals, and validation gates.
  Validation: Workstream docs exist and agree.
  Evidence: `docs/workstreams/api-module-split/DESIGN.md`
  Handoff: Continue with the behavior-preserving module split.

## M1 - Behavior-Preserving Module Split

- [x] AMS-020 [owner=codex] [deps=AMS-010] [scope=crates/nako-api/src]
  Goal: Split the `nako-api` crate root into `public_client`, `admin`,
  `metadata_diagnostics`, and `extension` modules while preserving root-level
  re-exports and current callers.
  Validation: `cargo check -p nako-api --tests`; `cargo nextest run -p nako-api
  --no-fail-fast`.
  passed.
  Evidence: `crates/nako-api/src/lib.rs` is a thin facade over focused modules.
  Handoff: Continue with OpenAPI/SDK contract checks.

## M2 - Contract And Workspace Validation

- [x] AMS-030 [owner=codex] [deps=AMS-020] [scope=crates/nako-api,sdk/typescript,workspace,docs]
  Goal: Prove module movement did not change OpenAPI, SDK, public DTOs, or
  workspace behavior.
  Validation: `cargo check -p nako-api --examples`; `npm run check --prefix
  sdk/typescript`; `cargo check --workspace --tests`; `cargo nextest run
  --workspace --no-fail-fast`; `git diff --check`.
  passed.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: Recommend NFO Round Trip preservation as the next goal.
