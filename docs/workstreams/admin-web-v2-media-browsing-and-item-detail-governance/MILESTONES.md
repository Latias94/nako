# Admin Web V2 Media Browsing And Item Detail Governance - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

Status: Complete 2026-05-25.

Exit criteria:

- Scope, non-goals, route order, bridge policy, and gate set are explicit.
- The first executable task is chosen.
- The lane references the closed library-management parity split.

Primary evidence:

- `DESIGN.md`
- `TODO.md`
- `WORKSTREAM.json`

## M1 - Route/API Readiness And Bridge Plan

Status: Complete 2026-05-25.

Exit criteria:

- Current public/admin read routes for browse and item detail are audited.
- Public-read bridge names and unsafe-field exclusions are decided.
- Any required backend/Admin API gap is split before UI implementation.

Primary gates:

```bash
git diff --check
```

## M2 - Catalog Browse Route

Status: Complete 2026-05-25.

Exit criteria:

- `/catalog` is route-owned.
- Browse/search filters are URL-owned and safe.
- Rows can navigate to item detail or show a precise blocker.
- Fallback and unsafe-text tests pass.

Primary gates:

```bash
cd apps/admin-web
npm run check
npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts
```

## M3 - Item Detail Route

Status: Complete 2026-05-25.

Exit criteria:

- `/items/:itemId` is route-owned.
- The page shows safe item facts, source context, metadata/artwork/NFO/provider
  readiness, and support links.
- The page does not add playback-client or repair mutation behavior.

Primary gates:

```bash
cd apps/admin-web
npm run check
npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts
```

## M4 - Repair And Action Split

Status: Complete 2026-05-25.

Exit criteria:

- Item-scoped repair/action gaps are re-scored.
- Follow-ons are split into vertical workflows with validation and safety
  expectations.

Primary gates:

```bash
git diff --check
```

## M5 - Closeout

Status: Complete 2026-05-25.

Exit criteria:

- Fresh final evidence is recorded.
- Review has no blocking findings.
- Remaining work is either completed, deferred, or split.
- `WORKSTREAM.json` status reflects final state.
