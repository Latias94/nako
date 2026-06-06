# Evidence

## Summary

- Ran Product-Operator M1 `release-fast` technical preflight against `main`
  after the default `fast` ladder passed.
- `release-fast` passed, so no blocker implementation task was opened from this
  run.
- The run regenerated SDK/Admin Web contract artifacts and ended with
  `git diff --check`; no committed source drift remained.

## Verification

- Date: 2026-06-06 20:15 Asia/Shanghai.
- Command:
  `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode release-fast`
- Result: passed.
- Delegated gate:
  `scripts/release-gate.ps1 -Mode fast`
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed.
  - redaction inventory scan wrote 7250 matches to
    `target/release-gate/redaction-inventory.txt`.
  - `cargo check -p nako-db --tests` passed.
  - `cargo nextest run -p nako-db sqlite_managed_artwork_contract --no-fail-fast`
    passed: 6 tests passed, 174 skipped.
  - `cargo check -p nako-server --tests` passed.
  - `cargo check -p nako-api --tests` passed.
  - `cargo check -p nako-client --tests` passed.
  - `cargo check -p nako-client-protocol --tests` passed.
  - `cargo nextest run -p nako-api openapi --no-fail-fast`
    passed: 13 tests passed, 82 skipped.
  - `cargo nextest run -p nako-api sdk --no-fail-fast`
    passed: 13 tests passed, 82 skipped.
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast`
    passed: 8 tests passed, 87 skipped.
  - `cargo nextest run -p nako-client --no-fail-fast`
    passed: 12 tests passed.
  - `cargo nextest run -p nako-client-protocol --no-fail-fast`
    passed: 15 tests passed.
  - `cargo tree -p nako-client` passed.
  - `cargo tree -p nako-client-protocol` passed.
  - `npm run generate --prefix sdk/typescript` passed.
  - `npm run check --prefix sdk/typescript` passed.
  - `npm run generate:admin-api --prefix apps/admin-web` passed.
  - `npm run check --prefix apps/admin-web` passed.
  - Final `git diff --check` passed.
  - `cargo nextest run -p nako-api managed_artwork --no-fail-fast`
    passed: 12 tests passed, 83 skipped.
  - `cargo nextest run -p nako-server managed_artwork --no-fail-fast`
    passed: 13 tests passed, 642 skipped.
  - `cargo nextest run -p nako-server self_host_smoke --no-fail-fast`
    passed: 1 test passed, 654 skipped.

## Decision

The technical M1 release-fast preflight is green. Because no DB/API/SDK/Admin
contract/Admin Web/server self-host blocker was exposed, this task does not
open a follow-on implementation slice. The next evidence-driven step is an
explicit playback gate.
