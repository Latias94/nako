# Admin Web Addon Operations Evidence And Gates

Status: Completed
Last updated: 2026-05-22

## Planned Gates

### Rust / Contract

- `cargo fmt --all -- --check`
- `cargo run -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo check -p nako-api -p nako-server --tests`

### Admin Web

- `npm test -- --run src/adminApi` from `apps/admin-web`
- `npm test -- --run src/App.test.tsx` from `apps/admin-web`
- `npm run build` from `apps/admin-web`

### Addon Regression

- `cargo nextest run -p nako-server addons --no-fail-fast`

### Hygiene

- `git diff --check`
- search generated/UI fixtures for forbidden sensitive terms if Addon mock data
  changes: token values, bearer headers, resolved secrets, raw payloads,
  Source Locators, storage URIs, local paths, cache URIs, database URLs.

## Evidence Log

- AWAO-010 opened the workstream and updated top-level goal/roadmap/workstream
  docs.
- AWAO-020:
  - `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts`
  - `cargo fmt --all -- --check`
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast` — 5 passed,
    46 skipped.
- AWAO-030:
  - `npm ci` in `apps/admin-web` to restore local frontend dependencies.
  - `npm test -- --run src/adminApi` — 2 files passed, 7 tests passed.
- AWAO-040/AWAO-050:
  - `npm test -- --run src/adminApi` — 2 files passed, 8 tests passed.
  - `npm test -- --run src/App.test.tsx` — 1 file passed, 5 tests passed.
- AWAO-060 closeout:
  - `cargo fmt --all -- --check`
  - `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts`
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast` — 5 passed,
    46 skipped.
  - `cargo check -p nako-api -p nako-server --tests`
  - `cargo nextest run -p nako-server addons --no-fail-fast` — 39 passed,
    212 skipped.
  - `npm run check` in `apps/admin-web`
  - `npm test` in `apps/admin-web` — 3 files passed, 13 tests passed.
  - `npm run build` in `apps/admin-web`
  - `git diff --check`
