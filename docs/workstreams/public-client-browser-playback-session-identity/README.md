# Public Client Browser Playback Session Identity

Status: Active
Last updated: 2026-05-28

This lane fixes the WMLP playback follow-on: browser playback tickets can start
safe media URLs, but the web client cannot honestly heartbeat playback because
the browser ticket response does not expose a stable playback session identity.

## Authoritative Docs

- `DESIGN.md` - problem, scope, non-goals, and architecture direction.
- `CONTRACT.md` - frozen browser playback session identity contract.
- `CONTRACT_READINESS.md` - contract choices and readiness checks.
- `TODO.md` - executable task ledger.
- `EVIDENCE_AND_GATES.md` - validation commands and evidence log.
- `HANDOFF.md` - current state and next action.

## Current Execution Point

`PBSI-020` froze the browser-visible playback session identity contract.
Continue with `PBSI-030`, the server/API/SDK implementation slice.
