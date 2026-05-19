# Durable Job Recovery Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Required Gates

```bash
cargo fmt --all -- --check
cargo check -p taru-db --tests
cargo nextest run -p taru-db sqlite_store_marks_unfinished_jobs_failed_on_startup --no-fail-fast
cargo check -p taru-server --tests
cargo nextest run -p taru-server app::tests::startup --no-fail-fast
cargo check -p taru-catalog --tests
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence To Record

- Regression test for unfinished job recovery at the SQLite adapter seam.
- Regression test for startup recovery at the server startup workflow seam.
- Startup report field and log behavior.
- Confirmation that terminal jobs are not rewritten.
- Confirmation that no public client/API contract changed.
- Confirmation that the old search projection seam is removed or intentionally
  retained.

## Closeout Evidence

- `JobRepository::fail_unfinished_jobs` expresses startup recovery intent at
  the repository seam.
- `SqliteStore::fail_unfinished_jobs` originally marked queued/running jobs
  failed with a startup-stale error and left succeeded/failed jobs unchanged.
  The later ownership-lease lane narrows generic recovery to running jobs and
  preserves queued jobs.
- `ServerStartupWorkflow` runs unfinished job recovery after migration and
  records `ServerStartupReport::recovered_jobs`.
- `app_startup_marks_unfinished_jobs_failed` proves restart recovery through
  the server startup workflow.
- Removed the unused `rebuild_search_projection` helper and its now-dead
  snapshot search projection helper from `taru-catalog`.
- Public HTTP API, SDK, CLI, license boundaries, and job status wire shapes did
  not change.
- Validation:
  - `cargo fmt --all -- --check`: passed.
  - `cargo check -p taru-db --tests`: passed.
  - `cargo nextest run -p taru-db sqlite_store_marks_unfinished_jobs_failed_on_startup --no-fail-fast`: 1 test passed.
  - `cargo check -p taru-server --tests`: passed.
  - `cargo nextest run -p taru-server app::tests::startup --no-fail-fast`: 7 tests passed.
  - `cargo check -p taru-catalog --tests`: passed.
  - `cargo check --workspace --tests`: passed.
  - `cargo nextest run --workspace --no-fail-fast`: 288 tests passed.
  - `git diff --check`: passed.
