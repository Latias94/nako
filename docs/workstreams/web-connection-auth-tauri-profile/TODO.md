# Web Connection Auth Tauri Profile - TODO

Status: Active
Last updated: 2026-05-28

## M0 - Activation

- [x] WCAT-010 [owner=planner] [deps=WROP-050] [scope=docs/workstreams/web-connection-auth-tauri-profile]
  Goal: Activate connection/auth lane after route ownership closes.
  Validation: WROP complete.
  Evidence: WORKSTREAM.json
  Handoff: DONE. WROP is complete; next task is WCAT-020.

## M1 - Connection State

- [x] WCAT-020 [owner=Codex] [deps=WCAT-010] [scope=web/src]
  Goal: Create a tested connection state boundary for browser and Tauri modes.
  Validation: npm --prefix web run test && npm --prefix web run check.
  Evidence: connection tests.
  Handoff: DONE. Shared connection profile/session boundary is tested, and Public/Admin data sources delegate to it.

## M2 - Setup And Account Wiring

- [x] WCAT-030 [owner=Codex] [deps=WCAT-020] [scope=web/src/features/setup,web/src/features/account]
  Goal: Wire setup/account to connection state, auth/session state, and error flows.
  Validation: npm --prefix web run test && npm --prefix web run build.
  Evidence: setup/account route tests.
  Handoff: DONE. Setup writes connection profile/session state, and account selection writes selected user session state without persisting passwords.

## M3 - Tauri Profile Bridge

- [ ] WCAT-040 [owner=Codex] [deps=WCAT-030] [scope=web/src-tauri,web/src]
  Goal: Wire Tauri profile bootstrap/invoke path for safe local profile persistence.
  Validation: cargo test --manifest-path web/src-tauri/Cargo.toml && npm --prefix web run test && npm --prefix web run tauri -- build.
  Evidence: Tauri tests and web adapter tests.
  Handoff: READY.

## M4 - Closeout

- [ ] WCAT-050 [owner=planner] [deps=WCAT-040] [scope=docs/workstreams/web-connection-auth-tauri-profile]
  Goal: Close the connection/auth lane.
  Validation: npm --prefix web run test && npm --prefix web run build && npm --prefix web run tauri -- build.
  Evidence: EVIDENCE_AND_GATES.md
  Handoff: Activate `web-admin-live-wiring`.
