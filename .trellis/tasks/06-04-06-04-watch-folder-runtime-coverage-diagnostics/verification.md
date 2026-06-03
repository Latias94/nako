# Verification

Final gate evidence for watch-folder runtime coverage diagnostics.

## Commands

* `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts`
  - Pass.
* `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output web/src/api/admin/generated/contract.ts`
  - Pass.
* `cargo fmt --all`
  - Pass.
* `cargo check -p nako-api -p nako-server --tests`
  - Pass.
* `cargo nextest run -p nako-api admin_overview --no-fail-fast`
  - Pass: 1 test run, 1 passed.
* `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  - Pass: 6 tests run, 6 passed.
* `cargo nextest run -p nako-server watch_folder --no-fail-fast`
  - Pass: 10 tests run, 10 passed.
* `cargo nextest run -p nako-server admin_v1_overview --no-fail-fast`
  - Pass: 1 test run, 1 passed.
* `cargo fmt --all -- --check`
  - Pass.
* `git diff --check`
  - Pass; only Git CRLF conversion warnings were printed.
* `python ./.trellis/scripts/task.py validate .trellis/tasks/06-04-06-04-watch-folder-runtime-coverage-diagnostics`
  - Pass: `implement.jsonl` and `check.jsonl` valid.

## Review Notes

* The watcher startup predicate remains `realtime_monitor` plus local
  `StorageUri`; unsupported or disabled libraries are reported, not started.
* Admin overview exposes only library IDs, names, root schemes, redacted root
  references, typed coverage status, and safe reasons.
* No route, schema, public client contract, scan enqueue, or reconciliation job
  behavior changed.
