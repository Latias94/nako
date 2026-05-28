# Web V0 Copy-First TanStack Refactor - Milestones

Status: Active
Last updated: 2026-05-28

## M0 - Scope Freeze

Status: done.

Exit criteria:

- Copy-first direction is explicit.
- Next.js is accepted only as a short-lived import/bootstrap shell.
- TanStack/Tauri/static frontend target is explicit.

## M1 - Copied Product Baseline

Status: done.

Exit criteria:

- `repo-ref/nako-admin-web` has been copied into `web/`.
- The copied app has a recorded build result or a precise quarantine blocker.
- Current reusable Tauri/API assets are preserved or intentionally recreated.

## M2 - Runtime Safety

Status: done.

Exit criteria:

- No frontend-owned TMDB secret route remains in the release path.
- No Vercel runtime/analytics assumption remains.
- Unsupported feature controls are removed, disabled, or labelled fixture/planned.
- Tauri does not require a Next/Node server sidecar.

## M3 - TanStack Architecture

Exit criteria:

- Route ownership no longer depends on one large view-state component.
- Media/Admin/Setup/Account routes are deep-linkable.
- Route-level chunks exist or a measured blocker is documented.

## M4 - Nako API Reattachment

Exit criteria:

- Media routes consume Public Client API.
- Admin routes consume Admin API only.
- Shared UI is DTO-free.
- Generated Admin contract drift remains covered.

## M5 - Performance And Desktop

Exit criteria:

- Build output has bundle evidence.
- Large grids/tables have virtualization decisions.
- Tauri static package/build smoke is recorded.
- Native playback remains split.

## M6 - Closeout

Exit criteria:

- v0 feature inventory is mapped to live/fixture/planned/blocked/deferred.
- Final gates are recorded.
- Follow-ons are split.
