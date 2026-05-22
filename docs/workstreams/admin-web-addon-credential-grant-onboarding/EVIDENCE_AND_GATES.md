# Admin Web Addon Credential and Grant Onboarding Evidence and Gates

Status: Completed
Last updated: 2026-05-22

## Required Gates

Rust/API gates:

- `cargo fmt --all -- --check`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo check -p nako-api -p nako-server --tests`

Admin Web gates from `apps/admin-web`:

- `npm run check`
- `npm test`
- `npm run build`

Repository gate:

- `git diff --check`

## Evidence Log

- 2026-05-22: Workstream opened. Implementation evidence pending.
- 2026-05-22: AWACG-020 contract/data-source slice implemented. Evidence:
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast`;
  - `npm test -- --run src/adminApi/client.test.ts`;
  - `npm test -- --run src/adminApi/dataSource.test.ts`.
- 2026-05-22: AWACG-030 UI slice implemented. Evidence:
  - `npm test -- --run src/App.test.tsx`;
  - `npm run check`.
- 2026-05-22: AWACG-040 closeout gates passed:
  - `cargo fmt --all -- --check`;
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast`;
  - `cargo check -p nako-api -p nako-server --tests`;
  - `npm run check`, `npm test`, and `npm run build` in `apps/admin-web`;
  - `git diff --check`.

## Safety Evidence Required Before Close

- Raw Addon Tokens appear only in issue/rotation action results and one-time UI
  notices. Proven by focused Admin Web client/data-source/UI tests.
- `load()` data, mock fallback, list responses, docs examples, and install
  guide previews do not include raw token values. Proven by data-source
  redaction assertions.
- Grant replacement uses explicit accepted Addon Permission assignments.
- No lifecycle automation is introduced. The UI actions only call Admin API
  token/grant/status/health/diagnostic routes.
