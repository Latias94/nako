# Server Runtime Deepening Task Ledger

Status: Completed
Last updated: 2026-05-17

## Tasks

- [x] SRD-010 [owner=codex] [scope=docs/workstreams/server-runtime-deepening]
  Goal: Open M38 with startup workflow, runtime job execution, scope,
  non-goals, and gates.
  Validation: `git diff --check`.
  Handoff: Continue with startup workflow implementation.

- [x] SRD-020 [owner=codex] [deps=SRD-010] [scope=crates/nako-server/src/app]
  Goal: Add `ServerStartupWorkflow` and move startup side effects out of
  `NakoApp::new_with_store` while preserving behavior.
  Validation: `cargo nextest run -p nako-server app::tests::startup --no-fail-fast`
  passed with 6 tests.
  Handoff: Startup report should become the test surface.

- [x] SRD-030 [owner=codex] [deps=SRD-020] [scope=crates/nako-server/src/app/runtime.rs, crates/nako-server/src/app/jobs.rs, crates/nako-server/src/app/metadata.rs]
  Goal: Add a durable job execution helper to `RuntimeSupervisor` and migrate
  library scan, metadata refresh, and metadata maintenance background jobs.
  Validation: `cargo nextest run -p nako-server app::runtime --no-fail-fast`
  passed with 4 tests; `cargo nextest run -p nako-server
  app::tests::metadata --no-fail-fast` passed with 10 tests.
  Handoff: Do not migrate webhook, automation, addon, NFO, or playback runners
  in this first slice.

- [x] SRD-040 [owner=codex] [deps=SRD-030] [scope=docs]
  Goal: Update GOALS, ROADMAP, workstream index, and M38 evidence.
  Validation: `git diff --check` passed.
  Handoff: M39 should cover repository seam deepening.

- [x] SRD-050 [owner=codex] [deps=SRD-040] [scope=workspace]
  Goal: Close M38 with focused and workspace gates.
  Validation: `cargo fmt --all -- --check`, `cargo check -p nako-server
  --tests`, focused nextest gates, `cargo check --workspace --tests`, `cargo
  nextest run --workspace --no-fail-fast` with 284 tests passed, `git diff
  --check`.
  Handoff: Recommend M39 repository seam deepening.
