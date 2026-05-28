# Web Admin Live Wiring - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

Admin dashboard has a live/fixture seam. Connection/auth, Tauri profile, and
route ownership are now in place, so deeper Admin pages can move to live seams.

## Active Task

- Task ID: WALW-020
- Owner: Codex
- Status: READY
- Validation: `npm --prefix web run test && npm --prefix web run build`

## Next Recommended Action

- Wire libraries, users, scheduled tasks, logs, and settings read models through
  Admin API modules, keeping fixture fallback explicit.
