# Durable Job Recovery Task Ledger

Status: Completed
Last updated: 2026-05-17

## M0 - Scope And Evidence Freeze

- [x] DJR-010 [owner=codex] [deps=none] [scope=docs/workstreams/durable-job-recovery]
  Goal: Open M41 with problem, target state, non-goals, and evidence anchors.
  Validation: Workstream docs exist and agree.
  Evidence: `docs/workstreams/durable-job-recovery/DESIGN.md`
  Handoff: Continue with regression tests before implementation.

## M1 - Startup Recovery

- [x] DJR-020 [owner=codex] [deps=DJR-010] [scope=crates/taru-core/src/repository/jobs.rs,crates/taru-db/src/jobs.rs,crates/taru-db/src/tests.rs]
  Goal: Add a repository operation that fails unfinished queued/running jobs
  from a previous process. Superseded: later ownership-lease work preserves
  queued jobs and fails only running jobs.
  Validation: `cargo nextest run -p taru-db running_jobs_failed_on_startup --no-fail-fast`
  passed with 1 test.
  Evidence: `running_jobs_failed_on_startup` proves running jobs become failed,
  queued jobs are preserved, and terminal jobs are unchanged.
  Handoff: Completed; startup integration followed in DJR-030.

- [x] DJR-030 [owner=codex] [deps=DJR-020] [scope=crates/taru-server/src/app/startup.rs,crates/taru-server/src/app/tests/startup.rs]
  Goal: Run unfinished job recovery during startup and expose the count in
  `ServerStartupReport`.
  Validation: `cargo nextest run -p taru-server app::tests::startup --no-fail-fast`
  passed with 7 tests.
  Evidence: `app_startup_marks_unfinished_jobs_failed` proves restarted app
  fails stale jobs and reports `recovered_jobs`.
  Handoff: Completed; deeper runtime cancellation semantics can be considered
  later, but startup recovery is now in place.

## M2 - Obsolete Seam Cleanup

- [x] DJR-040 [owner=codex] [deps=DJR-030] [scope=crates/taru-catalog/src/lib.rs]
  Goal: Remove or shrink the unused `rebuild_search_projection` old wide
  repository seam.
  Validation: `cargo check -p taru-catalog --tests`
  passed.
  Evidence: `rg rebuild_search_projection crates` returns no matches.
  Handoff: `CatalogHydrationPort` lookup deepening remains a follow-on.

## M3 - Closeout

- [x] DJR-050 [owner=codex] [deps=DJR-040] [scope=workspace,docs]
  Goal: Update top-level docs and close M41 with focused and workspace gates.
  Validation: `cargo fmt --all -- --check`; `cargo check --workspace --tests`;
  `cargo nextest run --workspace --no-fail-fast`; `git diff --check`.
  Evidence: `EVIDENCE_AND_GATES.md` records the closeout gates.
  Handoff: Recommend M42 `CatalogHydrationPort` lookup deepening if no higher
  correctness issue appears.
