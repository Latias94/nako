# Admin Web V2 Users Access Readiness - Milestones

Status: Complete
Last updated: 2026-05-26

## M0 - Boundary Frozen

Exit criteria:

- User, Role, Library Access, and Single-Admin Mode terms match `CONTEXT.md`.
- ADR 0024 and ADR 0028 are reflected in the scope.
- No fake account or RBAC mutation surface is planned for the first slice.

## M1 - Admin API Summary

Exit criteria:

- `GET /admin/v1/access/summary` is documented and contract-generated.
- Server tests prove Single-Admin Mode, effective Media Library access, and
  redaction behavior.
- Public Client API contracts stay unchanged.

## M2 - Admin Web Route

Exit criteria:

- `/access` appears in Admin Web navigation.
- The page uses `AdminDataSource` and route-local query loading.
- Live, mock fallback, readiness, principal, and library access states render.
- Unsafe auth/token/path/URL/env material is not rendered.

## M3 - Closeout

Exit criteria:

- Focused Rust/API/Admin Web gates pass or are explicitly skipped with reason.
- Browser desktop and mobile smoke evidence is recorded.
- `WORKSTREAM.json`, `HANDOFF.md`, and `CLOSEOUT.md` describe the final state
  and follow-ons.
