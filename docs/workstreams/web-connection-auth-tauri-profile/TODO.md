# Web Connection Auth Tauri Profile - TODO

Status: Queued
Last updated: 2026-05-28

## M0 - Activation

- [ ] WCAT-010 [owner=planner] [deps=WROP-050] [scope=docs/workstreams/web-connection-auth-tauri-profile]
  Goal: Activate connection/auth lane after route ownership closes.
  Validation: WROP complete.
  Evidence: WORKSTREAM.json
  Handoff: Next task is WCAT-020.

## M1 - Connection State

- [ ] WCAT-020 [owner=Codex] [deps=WCAT-010] [scope=web/src]
  Goal: Create a tested connection state boundary for browser and Tauri modes.
  Validation: npm --prefix web run test && npm --prefix web run check.
  Evidence: connection tests.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M2 - Setup And Account Wiring

- [ ] WCAT-030 [owner=Codex] [deps=WCAT-020] [scope=web/src/features/setup,web/src/features/account]
  Goal: Wire setup/account to connection state, auth/session state, and error flows.
  Validation: npm --prefix web run test && npm --prefix web run build.
  Evidence: setup/account route tests.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M3 - Tauri Profile Bridge

- [ ] WCAT-040 [owner=Codex] [deps=WCAT-030] [scope=web/src-tauri,web/src]
  Goal: Wire Tauri profile bootstrap/invoke path for safe local profile persistence.
  Validation: cargo test --manifest-path web/src-tauri/Cargo.toml && npm --prefix web run test && npm --prefix web run tauri -- build.
  Evidence: Tauri tests and web adapter tests.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M4 - Closeout

- [ ] WCAT-050 [owner=planner] [deps=WCAT-040] [scope=docs/workstreams/web-connection-auth-tauri-profile]
  Goal: Close the connection/auth lane.
  Validation: npm --prefix web run test && npm --prefix web run build && npm --prefix web run tauri -- build.
  Evidence: EVIDENCE_AND_GATES.md
  Handoff: Activate `web-admin-live-wiring`.
