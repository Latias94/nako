# M1 PostgreSQL Evidence Run

## Commands

- 2026-06-06 21:02 +08:00, git `ab1dbf5c`:
  `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode postgres`
  - Raw command output was reviewed locally; key public summary is recorded below.
- 2026-06-06 21:07 +08:00, git `ab1dbf5c`:
  `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite all-contracts`
  - Raw command output was reviewed locally; key public summary is recorded below.
- 2026-06-06 21:12 +08:00:
  `Get-NetTCPConnection -LocalPort 55432 -ErrorAction SilentlyContinue`
  - No listener was reported on the temporary harness port after cleanup.

## Result

- Passed: M1 ladder `postgres` mode completed with exit code 0.
- Passed: delegated release-gate PostgreSQL suite ran
  `cargo nextest run -p nako-db postgres_managed_artwork_contract --run-ignored ignored-only --no-fail-fast`.
- Passed: managed-artwork PostgreSQL contract summary was
  `6 tests run: 6 passed, 174 skipped`.
- Passed: broader PostgreSQL all-contracts harness ran
  `cargo nextest run -p nako-db postgres_ --run-ignored ignored-only --no-fail-fast`.
- Passed: all-contracts PostgreSQL summary was
  `51 tests run: 51 passed (35 slow), 129 skipped`.
- Cleanup note: the all-contracts run emitted a `pg_ctl stop` timeout warning,
  but the harness returned exit code 0, removed `target/postgres-contract`, and
  port `55432` was not listening afterward.

## Classification

- Result classification: passed.
- Environment classification: local PostgreSQL 17 tooling was available, so this
  was not an environment skip and did not require `NAKO_TEST_POSTGRES_URL`.
- Coverage classification: `scripts/m1-release-ladder.ps1 -Mode postgres`
  currently delegates to `scripts/release-gate.ps1 -Mode postgres`, whose
  PostgreSQL gate calls `scripts/postgres-contract-harness.ps1 -Suite
  managed-artwork`. This matches the M1 ladder matrix's current `postgres` row.
- Broader release-candidate confidence: the explicit `all-contracts` harness was
  run separately and passed, proving the full `postgres_` ignored contract set
  against a temporary local PostgreSQL cluster.
- Follow-up classification: no implementation blocker was opened. If future M1
  release policy wants the ladder's `postgres` mode itself to run
  `all-contracts`, that should be a release-gate policy change rather than a DB
  correctness blocker.
