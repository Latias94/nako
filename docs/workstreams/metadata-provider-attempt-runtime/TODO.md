# Metadata Provider Attempt Runtime Task Ledger

Status: Completed
Last updated: 2026-05-17

## M0 - Scope And Evidence Freeze

- [x] MPAR-010 [owner=codex] [deps=none] [scope=docs/workstreams/metadata-provider-attempt-runtime]
  Goal: Open M44 with problem, target state, non-goals, task ledger, and
  validation gates.
  Validation: Workstream docs exist and agree.
  Evidence: `docs/workstreams/metadata-provider-attempt-runtime/DESIGN.md`
  Handoff: Continue with provider attempt extraction.

## M1 - Internal Provider Attempt Runtime

- [x] MPAR-020 [owner=codex] [deps=MPAR-010] [scope=crates/nako-metadata/src]
  Goal: Extract provider attempt execution and classification from
  `strategy.rs` into an internal Module while preserving current behavior.
  Validation: `cargo check -p nako-metadata --tests`;
  `cargo nextest run -p nako-metadata --no-fail-fast`.
  passed.
  Evidence: Existing metadata strategy tests pass through the thinner strategy
  Interface and provider attempt runtime owns search/fetch, attempt recording,
  skipped attempts, raw response construction, and error classification.
  Handoff: Completed; commit and catalog hydration orchestration intentionally
  remain strategy-level behavior.

## M2 - Strategy Locality And Compatibility

- [x] MPAR-030 [owner=codex] [deps=MPAR-020] [scope=crates/nako-metadata/src/strategy.rs]
  Goal: Make `MetadataStrategyExecutor::refresh_item` read as high-level
  workflow orchestration and preserve its public Interface.
  Validation: focused fake-port test for refresh/hydration ports; existing
  provider fallback/rate-limit tests.
  passed.
  Evidence: Strategy delegates registered-provider handling, provider
  search/fetch, attempt construction, and error classification.
  Handoff: Public API and database behavior remain unchanged.

## M3 - Validation And Closeout

- [x] MPAR-040 [owner=codex] [deps=MPAR-030] [scope=workspace,docs]
  Goal: Close M44 with focused and workspace gates.
  Validation: `cargo fmt --all -- --check`; `cargo check --workspace --tests`;
  `cargo nextest run --workspace --no-fail-fast`; `git diff --check`.
  passed.
  Evidence: `EVIDENCE_AND_GATES.md` and `docs/GOALS.md`.
  Handoff: Recommend the next goal among `nako-api` module split, typed VFS
  storage errors, and NFO Round Trip.
