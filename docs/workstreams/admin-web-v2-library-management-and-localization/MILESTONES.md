# Admin Web V2 Library Management And Localization - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

Exit criteria:

- Scope and non-goals are explicit.
- Existing Admin Web V2 and metadata-profile authority docs are linked.
- First executable task is chosen.
- Jellyfin/Plex parity gaps are translated into Nako route/workflow language.

Primary evidence:

- `docs/workstreams/admin-web-v2-library-management-and-localization/DESIGN.md`
- `docs/workstreams/admin-web-v2-library-management-and-localization/TODO.md`

## M1 - Library Detail Read Model And Route

Status: Complete 2026-05-25.

Exit criteria:

- `/libraries/:libraryId` exists and is route-owned.
- The route has loading, fallback, empty/not-found, and unsafe-text behavior.
- The existing `/libraries` list has a clear detail affordance.
- Library details do not expose raw roots, Source Locators, credentials, or
  local paths.

Primary gates:

```bash
cd apps/admin-web
npm run check
npm run test -- App.test.tsx adminApi/dataSource.test.ts
```

## M2 - Metadata Profile And Library Actions

Status: Complete 2026-05-25.

Exit criteria:

- Metadata profile read/update route usage is represented safely or deliberately
  kept read-only with a blocker.
- Scan/NFO actions are user-triggered only and have explicit confirmation or
  deferral notes.
- Command failures are actionable and do not render unsafe response bodies.

Primary gates:

```bash
cd apps/admin-web
npm run check
npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts
cd ../..
cargo nextest run -p nako-server admin_library_command_routes_queue_background_jobs
cargo nextest run -p nako-api admin_contract
```

## M3 - Admin Web Localization Foundation

Status: Complete 2026-05-25.

Exit criteria:

- Admin Web has a small `i18n` boundary.
- English and Simplified Chinese catalogs exist.
- App shell plus library management route visible copy uses message IDs.
- API enum/query values and redaction-sensitive diagnostics remain stable.

Primary gates:

```bash
cd apps/admin-web
npm run check
npm run test -- App.test.tsx
```

## M4 - Parity Gap Split

Status: Complete 2026-05-25.

Exit criteria:

- Remaining Jellyfin/Plex-style gaps are re-scored.
- Follow-ons are split by vertical workflow.
- `HANDOFF.md` names the next recommended lane.

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
- `WORKSTREAM.json` status reflects the final state.
