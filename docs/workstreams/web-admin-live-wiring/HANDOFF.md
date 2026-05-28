# Web Admin Live Wiring - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

Admin dashboard and the accepted deeper Admin read pages now have live/fixture
seams. Libraries, users, scheduled tasks, logs, and settings consume Admin API
read models through `web/src/api/admin/read-models-data-source.ts`. Accepted
library/user/settings mutations go through
`web/src/api/admin/mutations-data-source.ts` with confirmation, error, and
permission states.

## Active Task

- Task ID: WALW-040
- Owner: Codex
- Status: READY
- Validation: `npm --prefix web run test && npm --prefix web run build`

## Next Recommended Action

- Replace copied plugin fixture UI with a Nako Addon Manager first slice.
