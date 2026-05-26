# Admin Web V2 Settings Mutation Authority - Handoff

Status: Closed
Last updated: 2026-05-26

## Current State

The lane is closed. ASM-010 through ASM-050 are complete.

Verified current baseline:

- `/settings` is implemented as a read-only Admin Web V2 route.
- The route uses `AdminDataSource.loadSettings()`.
- The client calls `GET /admin/v1/system/config`.
- The server registers only the `GET` system-config diagnostics route.
- `config.rs` has no accepted write/update authority for global
  `NakoServerConfig`.
- `build_router()` copies auth and network config into middleware state.
- runtime services and semaphores are built from startup config.
- `nako-db` has no global settings repository.

Backend predecessor update:

- `docs/workstreams/admin-settings-configuration-authority/` is closed.
- The implemented route is `GET|PUT /admin/v1/settings/metadata/raw-cache`.
- The generated Admin contract includes `settingsMetadataRawCache`,
  `AdminUpdateMetadataRawCacheSettingsRequest`, and
  `AdminMetadataRawCacheSettingsResponse`.
- The response reports `source = configured|admin` and
  `effect = active|requires_restart`.
- PUT persists an Admin desired-state override and does not hot-apply the
  current process when the route reports `requires_restart`.

Admin Web closeout:

- `AdminApiClient` and `AdminDataSource` consume
  `GET|PUT /admin/v1/settings/metadata/raw-cache`.
- `/settings` loads raw-cache settings separately from redacted system-config
  diagnostics.
- Save controls appear only when the raw-cache settings response is live-backed.
- The UI requires prepare/confirm before PUT.
- Mock fallback is read-only and cannot report fake success.
- Success and error notices are visible.
- `requires_restart` is surfaced when the persisted desired state does not
  match the running process.
- Browser smoke covered desktop fallback, desktop live mocked GET/PUT, and
  mobile layout after the responsive CSS fix.

## Next Task

The settings mutation lane is done. Continue the broader Admin Web V2 goal with
users/permissions/Library Access, or open a separate settings authority
workstream for another explicitly scoped field group.

Do not add save controls for network/auth, worker budgets, provider runtime,
staging roots, playback roots, artwork roots, raw TOML, URLs, hosts, tokens,
credentials, or environment variable names without a new backend authority
decision and route.

## Suggested Commands

```powershell
cargo nextest run -p nako-server admin_v1_metadata_raw_cache_settings --no-fail-fast
cd apps/admin-web
npm run test -- adminApi/client.test.ts adminApi/dataSource.test.ts App.test.tsx
npm run check
npm run build
git diff --check
```

## Residual Risks

- Network settings are high-value but sensitive; do not expose raw endpoints,
  origins, trusted proxy sources, or tunnel token references.
- Worker budget settings may be safer to edit, but live runtime resize may be
  unsupported.
- Full config persistence may need a separate architecture lane if TOML remains
  the startup source of truth.
- PostgreSQL admin settings contract remains unverified locally until
  `NAKO_TEST_POSTGRES_URL` is available.
- Admin Web build still warns that the main chunk exceeds 500 kB; split this in
  a separate frontend performance lane.
