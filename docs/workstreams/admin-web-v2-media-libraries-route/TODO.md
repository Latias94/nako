# Admin Web V2 Media Libraries Route - TODO

Status: Closed
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

- [x] AWVL-010 [owner=planner] [deps=none] [scope=docs/workstreams/admin-web-v2-media-libraries-route]
  Goal: Freeze the read-only Media Libraries route target, non-goals, data
  boundary, and evidence gates.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json,
  and HANDOFF.md exist and agree.
  Evidence: `docs/workstreams/admin-web-v2-media-libraries-route/DESIGN.md`
  Handoff: The first executable task is AWVL-020.

## M1 - Read-Only Route Migration

- [x] AWVL-020 [owner=codex] [deps=AWVL-010] [scope=apps/admin-web/src/App.tsx,apps/admin-web/src/adminApi,apps/admin-web/src/features/libraries]
  Goal: Replace the `/libraries` placeholder with a route-first read-only Media
  Libraries page backed by Admin system config diagnostics and deterministic
  fallback.
  Validation: `npm run check && npm run test -- --runInBand=false`
  Review: route module must keep Admin API access behind `AdminDataSource` and
  must not render storage roots, raw paths, tokens, passwords, or secret values.
  Evidence: `apps/admin-web/src/features/libraries/LibrariesPage.tsx`
  Handoff: DONE. `/libraries` now renders a route-first read-only page from
  Admin system config diagnostics.

## M2 - Validation And Browser Evidence

- [x] AWVL-030 [owner=codex] [deps=AWVL-020] [scope=apps/admin-web,docs/workstreams/admin-web-v2-media-libraries-route]
  Goal: Add route/data-source/redaction tests and record fresh frontend plus
  browser evidence.
  Validation: `npm run generate:admin-api`, `npm run check`, `npm run test`,
  `npm run build`, `git diff --check`, Playwright smoke for desktop and mobile.
  Review: review-workstream before accepting lane completion.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. Frontend gates, browser smoke, and redaction scans are recorded
  in `EVIDENCE_AND_GATES.md`.

## M3 - Closeout

- [x] AWVL-040 [owner=planner] [deps=AWVL-030] [scope=docs/workstreams/admin-web-v2-media-libraries-route]
  Goal: Close the lane or create narrower follow-ons for metadata profile,
  inventory, scan, and NFO workflows.
  Validation: fresh final gate evidence is recorded in EVIDENCE_AND_GATES.md.
  Review: review-workstream has no blocking findings.
  Evidence: `WORKSTREAM.json`, `HANDOFF.md`
  Handoff: DONE. Metadata profile editing, scan/NFO actions, and richer
  inventory are deferred follow-ons.
