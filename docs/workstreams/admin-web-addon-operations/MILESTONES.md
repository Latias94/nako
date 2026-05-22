# Admin Web Addon Operations Milestones

Status: Completed
Last updated: 2026-05-22

## Milestone AWAO.0: Contract Baseline

Exit criteria:

- Workstream docs exist and agree on scope/non-goals.
- `docs/GOALS.md`, `docs/ROADMAP.md`, and `docs/workstreams/README.md`
  mention the active lane.
- Addon Operations Admin API route constants and DTOs are generated into
  `apps/admin-web/src/adminApi/generated/contract.ts`.

## Milestone AWAO.1: Frontend Data Seam

Exit criteria:

- `AdminApiClient` has typed Addon Operations methods.
- `createAdminDataSource` loads Addon data with safe mock fallback.
- UI receives an Addon-specific read model rather than raw wire DTOs.
- Tests cover route paths, fallback behavior, and redaction-sensitive fields.

## Milestone AWAO.2: Operator UI

Exit criteria:

- Admin Web has a visible Addons operations surface.
- The surface shows list/detail, lifecycle status, grants/tokens summary,
  health state, manifest surfaces, and resource diagnostic state.
- Enable/disable, health check, and resource-call diagnostic actions are wired
  through a narrow seam.
- Hosted pages are visibly external and untrusted.

## Milestone AWAO.3: Closeout

Exit criteria:

- Rust and frontend gates pass.
- Workstream docs are updated with evidence.
- Follow-ons are split if they are Addon Manager, marketplace, package
  lifecycle, or process supervision work.
