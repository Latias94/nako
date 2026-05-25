# Admin Web V2 Media Libraries Route - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

Exit criteria:

- The route target is explicitly read-only.
- Admin API data ownership is defined.
- Metadata profile, scan, NFO, and create/edit semantics are out of the first
  slice.
- First executable task is AWVL-020.

Primary evidence:

- `docs/workstreams/admin-web-v2-media-libraries-route/DESIGN.md`
- `docs/workstreams/admin-web-v2-media-libraries-route/TODO.md`

## M1 - Read-Only Route Migration

Exit criteria:

- `/libraries` renders a real V2 route.
- The route uses `AdminDataSource` and live/mock section fallback.
- The page shows configured Media Library diagnostics without unsafe roots or
  credentials.
- `/legacy` remains available until later workflow parity is reached.

Primary gates:

- `npm run check`
- `npm run test`

## M2 - Validation And Browser Evidence

Exit criteria:

- Route tests cover data-source success, fallback, and redaction.
- Desktop and mobile smoke checks show no incoherent overlap or document-level
  horizontal overflow.
- Generated Admin API contract remains unchanged unless backend contract work is
  intentionally included.

Primary gates:

- `npm run generate:admin-api`
- `npm run check`
- `npm run test`
- `npm run build`
- `git diff --check`
- Playwright desktop/mobile smoke

## M3 - Closeout

Exit criteria:

- Evidence is recorded with dates and outcomes.
- Residual Media Libraries work is split into follow-ons or deferred.
- `WORKSTREAM.json` status reflects the lane state.

Closeout result: complete on 2026-05-25.
