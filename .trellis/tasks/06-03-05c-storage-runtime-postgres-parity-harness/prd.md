# Storage Runtime PostgreSQL Parity Harness

## Goal

Add focused PostgreSQL runtime parity evidence for storage admission and staging
behavior so future storage lanes do not rely only on SQLite or process-local
tests.

## MVP Scope

- Audit existing storage backend health, staging manifest, and job lease
  contract tests.
- Add the smallest parity harness or test coverage that can run against the
  repository's existing PostgreSQL test setup.
- Prefer contract tests or narrowly scoped server integration tests over a broad
  runtime rewrite.
- Document any unavailable PostgreSQL environment prerequisite clearly.

## Out of Scope

- No new storage admission feature behavior unless a parity bug is found.
- No scheduler fairness change; lane B owns that.
- No Admin/Web UI work.
- No schema migration unless the parity gap proves one is required.

## Acceptance Criteria

- [ ] PostgreSQL parity coverage exists for the selected storage runtime path,
  or evidence records why the local environment cannot run it.
- [ ] SQLite coverage remains passing.
- [ ] Tests prove storage admission/staging behavior has the same contract
  across backends where applicable.
- [ ] Follow-ons are recorded for broader PostgreSQL runtime harness work.

## Suggested Gates

- Focused `cargo nextest run -p nako-db <contract filter> --no-fail-fast`
- `cargo check -p nako-db -p nako-server --tests`
- Any existing PostgreSQL-gated command documented by the repo
- `cargo fmt --all -- --check`
- `git diff --check`
