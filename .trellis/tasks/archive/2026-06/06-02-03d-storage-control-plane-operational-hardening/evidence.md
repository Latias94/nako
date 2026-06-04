# Storage Control Plane Operational Hardening Evidence

Date: 2026-06-02
Selected slice: Admin storage staging pressure diagnostics.

## Selection

Chose the bounded staging pressure diagnostics slice because storage staging
already had records, cleanup, and VFS cache diagnostics, but operators could not
see aggregate pressure across manifest records without inspecting raw records.
The slice adds typed pressure state and counts to the Admin storage staging
diagnostics response.

## Boundaries Preserved

- No schema changes.
- No Public Client API, Public OpenAPI, or SDK route change.
- No playback runtime contract change.
- No raw Source Locator, Source Fingerprint, etag, local path, credential, or
  backend error is exposed in the new diagnostic summary.
- Generated Admin TypeScript contract copies were updated for `apps/admin-web`
  and `web`.

## Validation

- Synced current `main` into the worktree with `git merge --no-edit main`; no
  conflicts.
- `cargo fmt --all -- --check` passed.
- `cargo nextest run -p nako-api admin_contract storage --no-fail-fast` passed:
  18 tests.
- `cargo nextest run -p nako-server storage_staging --no-fail-fast` passed:
  2 tests.
- `cargo check -p nako-api -p nako-server --tests` passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed:
  6 tests.
- `npm --prefix apps/admin-web run check` passed.
- `npm --prefix apps/admin-web run test` passed: 6 files, 160 tests.
- `npm run check --prefix web` passed.
- `npm run test --prefix web -- src/test/data-source-contracts.test.ts`
  passed: 1 file, 43 tests.

## Follow-ons

- Source Fingerprint escalation remains a separate storage/library follow-on.
- Scan scheduling and PostgreSQL runtime harness remain separate control-plane
  follow-ons.
- Playback artifact/source-read pressure can build on this aggregate staging
  diagnostic but was intentionally not coupled to this Admin DTO slice.
