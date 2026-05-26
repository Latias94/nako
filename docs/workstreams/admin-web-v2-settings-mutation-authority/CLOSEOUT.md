# Admin Web V2 Settings Mutation Authority - Closeout

Date: 2026-05-26
Status: Closed

## Outcome

This lane shipped the first safe Admin Web V2 settings mutation slice:
metadata raw cache retention and startup cleanup.

The backend authority is:

- `GET /admin/v1/settings/metadata/raw-cache`
- `PUT /admin/v1/settings/metadata/raw-cache`

The UI authority is:

- `apps/admin-web/src/features/settings/SettingsPage.tsx`
- `apps/admin-web/src/adminApi/client.ts`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/adminApi/generated/contract.ts`

Admin Web now loads raw-cache settings separately, enables mutation controls
only for live Admin API data, requires prepare/confirm before PUT, renders
success and failure states, and surfaces `requires_restart` when the persisted
Admin desired state differs from the active process.

## Gates

Fresh closeout evidence is recorded in `EVIDENCE_AND_GATES.md`.

Passed:

- `cargo nextest run -p nako-server admin_metadata_raw_cache_settings --no-fail-fast`
- `cargo nextest run -p nako-server admin_v1_metadata_raw_cache_settings --no-fail-fast`
- `cargo nextest run -p nako-db sqlite_admin_settings_contract_metadata_raw_cache_settings_round_trip --no-fail-fast`
- `cargo test -p nako-api admin_contract_includes_route_constants -- --nocapture`
- `cargo test -p nako-api admin_web_generated_contract_matches_generator_output -- --nocapture`
- `cargo fmt --all --check`
- `cd apps/admin-web && npm run generate:admin-api`
- `cd apps/admin-web && npm run test -- adminApi/client.test.ts adminApi/dataSource.test.ts App.test.tsx`
- `cd apps/admin-web && npm run check`
- `cd apps/admin-web && npm run build`
- `cd apps/admin-web && npm run test`
- `git diff --check`
- Playwright CLI desktop fallback smoke for `/settings`
- Playwright CLI desktop live mocked GET/PUT smoke for `/settings`
- Playwright CLI mobile 390x844 layout smoke for `/settings`

Skipped:

- PostgreSQL admin settings repository contract, because
  `NAKO_TEST_POSTGRES_URL` is not configured in this environment.

## Review

No blocking review findings remain.

Workstream compliance:

- The shipped mutation surface is limited to the ASCA-authorized metadata raw
  cache slice.
- The UI does not expose raw TOML, URLs, hosts, filesystem paths, roots, env var
  names, credentials, tokens, provider secrets, or raw provider config.
- Mock fallback cannot fake a settings save.
- Public Client API contracts are unchanged.

Code quality:

- Admin Web uses `AdminDataSource` and `AdminApiClient`; `SettingsPage` does
  not issue direct fetches.
- Tests cover client route shape, data-source fallback behavior, confirm before
  save, mutation success, mutation failure, and mock fallback disablement.
- Browser smoke confirmed desktop behavior and mobile layout.

## Follow-Ons

- Run the ignored PostgreSQL contract once `NAKO_TEST_POSTGRES_URL` is
  available.
- Split a new settings authority lane before editing any other global settings
  group.
- Address the Admin Web production bundle size warning in a separate frontend
  performance lane.
- Continue the broader Admin Web V2 goal with users/permissions/Library Access.
