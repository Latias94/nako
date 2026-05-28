# Web Connection Auth Tauri Profile - Evidence And Gates

Status: Complete
Last updated: 2026-05-28

## Gate Set

```bash
npm --prefix web run test
npm --prefix web run check
npm --prefix web run build
cargo test --manifest-path web/src-tauri/Cargo.toml
npm --prefix web run tauri -- build
git diff --check
```

Security scans should confirm no bearer/session secrets are placed in URLs,
logs, shared UI props, or persisted profile fields.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-28 | WCAT-010 | Queued as lane 4 after route-owned product surfaces. | Queued. |
| 2026-05-28 | WCAT-010 | Activated after WROP closed with route/state tests, check, full test, build, and diff gate passing. | Active. Current task is WCAT-020. |
| 2026-05-28 | WCAT-020 | Added shared connection profile/session boundary, migrated Public/Admin connection loaders to it, and verified bearer tokens stay out of persisted profile storage. Ran `npm --prefix web run test -- src/test/connection-profile.test.ts src/test/data-source-contracts.test.ts`, `npm --prefix web run check`, `npm --prefix web run test`, `npm --prefix web run build`, and scoped `git diff --check`. | Passed. Current task is WCAT-030. |
| 2026-05-28 | WCAT-030 | Wired setup connection testing and account selection to the shared connection/session boundary. Added setup/account connection tests. Ran `npm --prefix web run test -- src/test/setup-account-connection.test.tsx src/test/connection-profile.test.ts`, `npm --prefix web run check`, `npm --prefix web run test`, `npm --prefix web run build`, and scoped `git diff --check`. | Passed. Current task is WCAT-040. |
| 2026-05-28 | WCAT-040 | Added Tauri local profile file persistence, fixed a profile mutex deadlock, added web invoke adapter/tests, and verified Tauri packaging. Ran `npm --prefix web run check`, `npm --prefix web run test -- src/test/tauri-profile.test.ts src/test/connection-profile.test.ts`, `npm --prefix web run test`, `npm --prefix web run build`, `cargo test --manifest-path web/src-tauri/Cargo.toml`, `npm --prefix web run tauri -- build`, and scoped `git diff --check`. | Passed. Current task is WCAT-050. |
| 2026-05-28 | WCAT-050 | Closed lane after browser tests/build, Tauri Rust tests, Tauri build, and secret persistence checks passed. | Complete. Activate WALW. |
