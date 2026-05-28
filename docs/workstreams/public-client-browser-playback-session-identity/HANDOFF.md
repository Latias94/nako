# Public Client Browser Playback Session Identity - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

This lane is open. WMLP closed with browser-ticket playback wired in `web/`, but
heartbeat is blocked because the ticket response does not expose a stable
playback session id.

## Active Task

- Task ID: PBSI-020
- Owner: Codex
- Status: READY
- Validation: contract freeze, protocol/API evidence, formatting, and diff
  check.

## Next Recommended Action

Start PBSI-020. Prefer a Public Client JSON response field for session identity
instead of teaching the web client to mine media response headers.
