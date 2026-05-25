# Admin Web V2 Acquisition Intake Route Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

Exit criteria:

- Workstream docs exist and agree on read-only route scope.
- Backend mutation workflows are explicitly out of scope.

## M1 - Route-First Intake Page

Exit criteria:

- `/acquisition/intake` is reachable from V2 navigation.
- Search params map to `AdminAcquisitionIntakeCandidatesQuery`.
- The route uses route-local live/mock fallback and does not depend on
  `LegacyDashboard`.
- Rendering uses only safe candidate summary fields.

## M2 - Evidence And Browser Smoke

Exit criteria:

- Admin API generation, TypeScript check, tests, build, and diff hygiene pass.
- Desktop and mobile smoke prove the route is nonblank, responsive, and free of
  unsafe rendered text.

## M3 - Closeout

Exit criteria:

- TODO marks all tasks complete.
- `WORKSTREAM.json` status is completed.
- `HANDOFF.md` and `CLOSEOUT.md` record residual risks and follow-ons.
