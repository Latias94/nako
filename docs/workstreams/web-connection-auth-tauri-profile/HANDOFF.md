# Web Connection Auth Tauri Profile - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

Route ownership is complete. Setup/account are still fixture/planned and can now
be wired to shared connection and auth state.

## Active Task

- Task ID: WCAT-030
- Owner: Codex
- Status: DONE
- Validation: `npm --prefix web run test -- src/test/setup-account-connection.test.tsx src/test/connection-profile.test.ts`, `npm --prefix web run check`, `npm --prefix web run test`, `npm --prefix web run build`, and scoped `git diff --check`.

## Next Recommended Action

- Start WCAT-040 by wiring the Tauri profile bootstrap/invoke path for safe local profile persistence.
