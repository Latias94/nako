# Admin Web V2 Item Artwork Selection - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

Status: Complete 2026-05-25.

Exit criteria:

- Lane scope, non-goals, route/API order, and gate set are explicit.
- First executable task is selected.
- The lane references GAR closeout and the MBG artwork follow-on split.

## M1 - Route/API Contract Readiness

Status: Complete 2026-05-25.

Exit criteria:

- Backend route inventory for item artwork gallery/select/unpublish is audited.
- Generated Admin Web contract gaps are explicit.
- Request/response DTOs and redaction expectations are accepted or split.

Primary gate:

```bash
git diff --check
```

## M2 - Generated Contract And API Bridge

Status: Complete 2026-05-25.

Exit criteria:

- Generated Admin Web route constants and DTOs cover gallery/select/unpublish.
- `AdminApiClient` methods call generated routes with encoded item/artifact
  IDs and image kinds.
- Client tests prove request paths, methods, and bodies.

Primary gate:

```bash
cd apps/admin-web
npm run generate:admin-api
npm run check
npm run test -- adminApi/client.test.ts
```

## M3 - Artwork Gallery UI

Status: Complete 2026-05-25.

Exit criteria:

- One Media Item can open a safe artwork gallery.
- Candidate, artifact, and Selected Artwork rows use redacted summaries.
- Redaction and fallback tests pass.

Primary gate:

```bash
cd apps/admin-web
npm run check
npm run test -- App.test.tsx adminApi/dataSource.test.ts
```

## M4 - Confirmed Select And Unpublish Actions

Status: Complete 2026-05-25.

Exit criteria:

- Select/replace requires explicit confirmation.
- Unpublish requires explicit confirmation.
- Mutation result rendering is redaction-safe.
- Mutation tests cover request body, success, idempotent/replay-like state,
  error, and no fake mutation fallback.

Primary gate:

```bash
cd apps/admin-web
npm run check
npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts
```

## M5 - Verification And Browser Smoke

Status: Complete 2026-05-25.

Exit criteria:

- Focused and full Admin Web gates pass.
- Relevant Rust/Admin contract gates pass when contract source changes.
- Desktop/mobile browser smoke covers item detail, artwork gallery, select
  confirmation, and unpublish confirmation paths.
- Unsafe text checks pass.

## M6 - Closeout

Status: Complete 2026-05-25.

Exit criteria:

- Fresh final evidence is recorded.
- Review has no blocking findings.
- Remaining work is either completed, deferred, or split.
- `WORKSTREAM.json` status reflects final state.

Primary gates:

```bash
cd apps/admin-web && npm run check
cd apps/admin-web && npm run test
cd apps/admin-web && npm run build
cargo nextest run -p nako-api admin_contract
cargo fmt --all --check
git diff --check
```
