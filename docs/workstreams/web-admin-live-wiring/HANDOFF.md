# Web Admin Live Wiring - Handoff

Status: Complete
Last updated: 2026-05-28

## Current State

Admin dashboard and the accepted deeper Admin read pages now have live/fixture
seams. Libraries, users, scheduled tasks, logs, and settings consume Admin API
read models through `web/src/api/admin/read-models-data-source.ts`. Accepted
library/user/settings mutations go through
`web/src/api/admin/mutations-data-source.ts` with confirmation, error, and
permission states.
The copied plugin fixture page has been replaced by the first Nako Addon
Manager slice backed by `web/src/api/admin/addons-data-source.ts`.

## Active Task

- Task ID: WALW-050
- Owner: Codex
- Status: DONE
- Validation: `npm --prefix web run test`, `npm --prefix web run check`, `npm --prefix web run build`, `git diff --check`

## Next Recommended Action

- Activate `web-bundle-budget-and-product-pruning`.
