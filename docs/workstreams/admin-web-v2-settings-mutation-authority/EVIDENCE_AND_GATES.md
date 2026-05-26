# Admin Web V2 Settings Mutation Authority - Evidence And Gates

Status: Closed
Last updated: 2026-05-26

## Smallest Current Repro

```powershell
rg -n "system/config|update.*config|NetworkAccessConfig|SettingsPage|loadSettings" crates/nako-server/src crates/nako-api/src apps/admin-web/src docs/api/HTTP_API.md
git diff --check
```

This proves the current settings route/API baseline is diagnostic-only and
keeps the first task grounded in current code instead of recovered session
memory.

## Gate Set

### ASM-020 Readiness Gate

```powershell
rg -n "system/config|update.*config|NetworkAccessConfig|SettingsPage|loadSettings" crates/nako-server/src crates/nako-api/src apps/admin-web/src docs/api/HTTP_API.md
git diff --check
```

Record what fields are actually editable and what fields remain read-only.

### Rust API Gate

```powershell
cargo nextest run -p nako-server <settings-mutation-test-filter> --no-fail-fast
cargo test -p nako-api admin_contract_includes_route_constants -- --nocapture
cargo fmt --all --check
```

Use after ASM-030 adds or changes Admin API routes.

### Admin Web Gate

```powershell
cd apps/admin-web
npm run generate:admin-api
npm run check
npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts
npm run test
npm run build
```

Use after ASM-040 adds controls to `/settings`.

### Browser Smoke Gate

Use Playwright against `/settings` at desktop and mobile viewports. Verify:

- mutation controls exist only for live-backed settings;
- fallback/mock mode cannot save;
- confirmation, success, failure, and stale states are visible;
- no unsafe config text appears;
- no horizontal overflow or incoherent overlap.

### Whitespace Gate

```powershell
git diff --check
```

## Evidence Log

| Date | Task | Command or artifact | Result |
| --- | --- | --- | --- |
| 2026-05-25 | ASM-010 | `docs/workstreams/admin-web-v2-settings-mutation-authority/*` | Opened lane and froze diagnostic-only baseline. |
| 2026-05-25 | ASM-020 | `rg -n "system/config\|update.*config\|NetworkAccessConfig\|SettingsPage\|loadSettings" crates/nako-server/src crates/nako-api/src apps/admin-web/src docs/api/HTTP_API.md`; code inspection of `config.rs`, `app/composition.rs`, `http.rs`, `http/network.rs`, and `nako-db` repositories | DONE_WITH_CONCERNS. Current system settings are diagnostic-only; global settings mutation needs backend configuration authority first. Opened `docs/workstreams/admin-settings-configuration-authority/`. |
| 2026-05-26 | ASM-030 | `docs/workstreams/admin-settings-configuration-authority/CLOSEOUT.md`; `cargo test -p nako-api admin_web_generated_contract_matches_generator_output -- --nocapture`; `git diff --check` | DONE_WITH_CONCERNS. Backend predecessor closed with `GET|PUT /admin/v1/settings/metadata/raw-cache` and generated Admin contract. Admin Web mutation UI may continue only for metadata raw cache settings and must surface restart-required state. |
| 2026-05-26 | ASM-030 | `cd apps/admin-web && npm run generate:admin-api` | Pass. Regenerated Admin Web contract from `nako-api`; `settingsMetadataRawCache`, request DTO, and response DTO remain generated. |
| 2026-05-26 | ASM-030 | `cargo nextest run -p nako-server admin_metadata_raw_cache_settings --no-fail-fast` | Pass. Startup composition persists and reloads metadata raw cache Admin override across restart. |
| 2026-05-26 | ASM-030 | `cargo nextest run -p nako-server admin_v1_metadata_raw_cache_settings --no-fail-fast` | Pass. Admin HTTP route round-trips persisted override and rejects zero retention. |
| 2026-05-26 | ASM-030 | `cargo nextest run -p nako-db sqlite_admin_settings_contract_metadata_raw_cache_settings_round_trip --no-fail-fast` | Pass. SQLite repository contract round-trips the single raw-cache Admin override record. |
| 2026-05-26 | ASM-030 | `cargo test -p nako-api admin_contract_includes_route_constants -- --nocapture` | Pass. Admin API route constants include the settings raw-cache route. |
| 2026-05-26 | ASM-030 | `cargo test -p nako-api admin_web_generated_contract_matches_generator_output -- --nocapture` | Pass. Checked-in generated Admin Web contract matches the generator output. |
| 2026-05-26 | ASM-040 | `cd apps/admin-web && npm run test -- adminApi/client.test.ts adminApi/dataSource.test.ts App.test.tsx` | Pass. 3 files, 123 tests. Covers Admin API client/dataSource raw-cache GET/PUT, no fake mutation fallback, confirm-before-save, success effect, and visible save error. |
| 2026-05-26 | ASM-040 | `cd apps/admin-web && npm run check` | Pass. TypeScript project build succeeds. |
| 2026-05-26 | ASM-040 | `cd apps/admin-web && npm run build` | Pass with warnings. Vite built production assets; warnings were `rolldown:vite-resolve` plugin timing and chunk size over 500 kB. |
| 2026-05-26 | ASM-040 | Playwright CLI `/settings` desktop 1440x1000, fallback Admin API | Pass. Mock fallback renders raw-cache controls disabled; no fake save action is available. Browser console has 0 errors. |
| 2026-05-26 | ASM-040 | Playwright CLI `/settings` desktop 1440x1000 with raw-cache route mocked live | Pass. `Edit override -> Prepare save -> Confirm save` sends `PUT /admin/v1/settings/metadata/raw-cache`, then shows `requires_restart` and saved notice. |
| 2026-05-26 | ASM-040 | Playwright CLI `/settings` mobile 390x844 after responsive CSS fix | Pass. Raw-cache header, badges, fields, facts, and buttons stay inside the content width with no incoherent overlap. |
| 2026-05-26 | ASM-050 | `cd apps/admin-web && npm run test` | Pass. 4 files, 125 tests. |
| 2026-05-26 | ASM-050 | `cargo fmt --all --check` | Pass. |
| 2026-05-26 | ASM-050 | `git diff --check` | Pass with Git LF-to-CRLF working-copy warnings only. |
| 2026-05-26 | ASM-050 | `python -m json.tool docs\workstreams\admin-web-v2-settings-mutation-authority\WORKSTREAM.json > $null` | Pass after closeout update. `WORKSTREAM.json` remains valid JSON with status `closed`. |
| 2026-05-26 | ASM-050 | `if ($env:NAKO_TEST_POSTGRES_URL) { ... } else { ... }` | Skipped. `NAKO_TEST_POSTGRES_URL` is missing, so the ignored PostgreSQL admin settings contract was not run locally. |

## Evidence Anchors

- `docs/workstreams/admin-web-v2-settings-mutation-authority/DESIGN.md`
- `docs/workstreams/admin-web-v2-settings-mutation-authority/TODO.md`
- `docs/workstreams/admin-web-v2-settings-mutation-authority/ROUTE_API_READINESS.md`
- `apps/admin-web/src/features/settings/SettingsPage.tsx`
- `apps/admin-web/src/adminApi/client.ts`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/config.rs`
- `docs/api/HTTP_API.md`

## Review Notes

Self-review before closeout:

- Workstream compliance: no blocking findings. The lane shipped only the
  ASCA-authorized raw-cache settings slice and did not add broad config,
  secret, URL, path, env var, token, or credential editing.
- Code quality: no blocking findings. The Settings route uses
  `AdminDataSource`, the client and generated contract own route calls, mock
  fallback does not fake mutation success, and tests exercise public UI and API
  seams.
- Important residual risk: the PostgreSQL repository contract exists but was
  not run because this environment lacks `NAKO_TEST_POSTGRES_URL`.
- Minor follow-on: production build still warns about a large Admin Web bundle;
  this predates the raw-cache slice and should be handled by a separate
  code-splitting pass.
