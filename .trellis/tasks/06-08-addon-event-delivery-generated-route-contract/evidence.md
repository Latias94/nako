# Evidence

## 2026-06-08

- [OK] `npm run check --prefix apps/admin-web`
- [OK] `npm run test --prefix apps/admin-web -- adminApi/client.test.ts adminApi/dataSource.test.ts App.test.tsx` (179 passed)
- [OK] `cargo check -p nako-api --tests`
- [OK] `cargo check -p nako-server --tests`
- [OK] `cargo nextest run -p nako-api admin_contract --no-fail-fast` (8 passed)
- [OK] `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast` (1 passed)
- [OK] `cargo nextest run -p nako-server addon_event --no-fail-fast` (13 passed)
- [OK] `cargo fmt --all -- --check`
- [OK] `git diff --check` (line-ending warnings only)
- [OK] `python ./.trellis/scripts/task.py validate 06-08-addon-event-delivery-generated-route-contract`
