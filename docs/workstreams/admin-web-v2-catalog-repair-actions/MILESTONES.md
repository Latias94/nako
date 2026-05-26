# Admin Web V2 Catalog Repair Actions - Milestones

Status: Complete
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

Status: Complete 2026-05-25.

Exit criteria:

- Lane scope, non-goals, repair-readiness order, and gate set are explicit.
- First executable task is selected.
- The lane references MBG, GAR, AWA, ADR 0027, and the M60 read model.

## M1 - Route/API Repair Readiness

Status: Complete 2026-05-25.

Exit criteria:

- Existing Catalog Governance list/detail/action routes are inventoried.
- Generated Admin Web contract gaps are explicit.
- First repair action and redaction policy are accepted or split.

Primary gate:

```bash
git diff --check
```

## M2 - Repair Detail And Review Plan

Status: Complete 2026-05-25.

Exit criteria:

- Redaction-safe Admin DTOs expose the context needed for the first action.
- Review-plan or dry-run route explains what will change before mutation.
- Backend/API tests prove redaction and route behavior.
- Generated Admin Web contract is synchronized when routes/DTOs change.

## M3 - Confirmed Repair Mutation

Status: Complete 2026-05-25.

Exit criteria:

- First mutation is item/action scoped and idempotent where applicable.
- Result DTOs are redaction-safe.
- Admin Web client/data-source wrappers do not fake mutation success.
- Failure states are test-covered.

## M4 - Admin Web Repair UI

Status: Complete 2026-05-25.

Exit criteria:

- Operators can open one Catalog Governance repair context.
- Review-plan context renders safely.
- Mutation requires explicit confirmation.
- Success, idempotent, pending, and failure states are visible.
- Unsafe text exclusions are test-covered.

## M5 - Verification And Browser Smoke

Status: Complete 2026-05-25.

Exit criteria:

- Focused and full Admin Web gates pass.
- Relevant Rust/Admin contract gates pass.
- Desktop/mobile browser smoke covers queue, repair context, confirmation,
  result, failure, overflow, console errors, and unsafe text exclusions.

## M6 - Closeout

Status: Complete 2026-05-25.

Exit criteria:

- Fresh final evidence is recorded.
- Review has no blocking findings.
- Remaining catalog repair breadth is completed, deferred, or split.
- `WORKSTREAM.json` status reflects final state.
