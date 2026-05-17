# Metadata Provider Attempt Runtime Milestones

Status: Completed
Last updated: 2026-05-17

## M44.0 - Workstream Baseline

Exit criteria:

- M44 is recorded as the active goal.
- The workstream records problem, target state, non-goals, and validation
  gates.
- The scope excludes provider breadth, public API, NFO, playback, and database
  schema work.

Status: Completed.

## M44.1 - Runtime Extraction

Exit criteria:

- Provider attempt execution has an internal Module.
- Attempt classification and skipped-provider handling are local to that
  Module.
- Existing provider success, no-match, fallback, disabled, unavailable, and
  rate-limit behavior still passes.

Status: Completed.

Expected gates:

- `cargo fmt --all -- --check`
- `cargo check -p taru-metadata --tests`
- `cargo nextest run -p taru-metadata --no-fail-fast`

## M44.2 - Strategy Compatibility

Exit criteria:

- `MetadataStrategyExecutor::refresh_item` public shape is unchanged.
- Strategy-level code reads as high-level refresh orchestration.
- Fake-port tests still prove refresh commit and catalog hydration usage
  without SQLite.

Status: Completed.

Expected gates:

- focused `cargo nextest run -p taru-metadata strategy::port_tests::refresh_service_uses_refresh_and_hydration_ports_without_sqlite --no-fail-fast`

## M44.3 - Closeout

Exit criteria:

- Workstream evidence is complete.
- `docs/GOALS.md` records M44 completion and recommends the next goal.
- Follow-on risks are carried forward without bloating M44.

Status: Completed.

Expected gates:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace --no-fail-fast`
- `git diff --check`
