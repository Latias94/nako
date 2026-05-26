# Admin Settings Configuration Authority - Handoff

Status: Closed
Last updated: 2026-05-26

## Current State

This lane was opened from
`admin-web-v2-settings-mutation-authority` ASM-020.

Verified baseline:

- Global server settings come from `NakoServerConfig` loaded at startup.
- No global settings repository or config save path exists.
- Network/auth middleware state is copied from config during `build_router()`.
- Runtime semaphores and services are built from config during
  `NakoAppComposition::build()`.

Implemented ASCA-020/ASCA-030 slice:

- first mutable field group:
  `metadata.raw_cache_retention_ms` and
  `metadata.maintenance.raw_cache_cleanup_on_startup`;
- SQLite/PostgreSQL migrations add a single-row Admin settings override table;
- `NakoAppComposition::build()` migrates before service construction and
  applies persisted Admin overrides during startup config merge;
- `GET /admin/v1/settings/metadata/raw-cache` reports configured/Admin source
  and active/restart-required effect;
- `PUT /admin/v1/settings/metadata/raw-cache` persists the override and
  validates `retention_ms > 0`;
- generated Admin Web contract includes the route and DTOs.

## Next Task

Return to `admin-web-v2-settings-mutation-authority` ASM-030/ASM-040.

Admin Web may build real controls only for the implemented metadata raw cache
field group and must surface the restart-required effect. Broader settings are
still out of scope.

## Handoff Back

After ASCA-030 lands a real Admin API route, return to
`admin-web-v2-settings-mutation-authority` ASM-030/ASM-040 for generated
contract and UI controls.

## Residual Risk

PostgreSQL adapter and ignored contract coverage were added, but the local
verification environment did not set `NAKO_TEST_POSTGRES_URL`, so PostgreSQL
runtime parity was not executed in this session.
