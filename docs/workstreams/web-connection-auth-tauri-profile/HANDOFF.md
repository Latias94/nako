# Web Connection Auth Tauri Profile - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

Route ownership is complete. Setup/account are still fixture/planned and can now
be wired to shared connection and auth state.

## Active Task

- Task ID: WCAT-040
- Owner: Codex
- Status: DONE
- Validation: `npm --prefix web run check`, `npm --prefix web run test -- src/test/tauri-profile.test.ts src/test/connection-profile.test.ts`, `npm --prefix web run test`, `npm --prefix web run build`, `cargo test --manifest-path web/src-tauri/Cargo.toml`, `npm --prefix web run tauri -- build`, and scoped `git diff --check`.

## Next Recommended Action

- Close WCAT-050, then activate `web-admin-live-wiring`.
