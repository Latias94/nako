# Client Surface And Access Product Architecture - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

This planning lane is open. CSAPA-010 drafted the product architecture for
Admin Web, Media Web, desktop, mobile, accounts, roles, Library Access, and
Management Context Links.

CSAPA-020 is complete through `identity-and-library-access-contract`. The
backend now has durable identity/access persistence, bootstrap administrator
semantics, Admin API access-management routes, and Public Client API
effective-access enforcement.

CSAPA-030 is complete through `media-web-client-foundation`. The Media Web
execution lane is now split and its first task is MWF-020 route/API readiness.

## Next Recommended Task

CSAPA-040: split the Management Context Links route/link matrix once Media Web
route readiness is known.

Recommended first decisions:

- define media-to-admin links from libraries, Media Items, source/version
  choice, and playback errors;
- define admin-to-media links from library detail and item detail;
- gate each link by Role plus Library Access;
- keep stable IDs and safe query params only;
- leave destructive or broad actions owned by Admin Web confirmation flows.

## Key Constraints

- Media Web must consume Public Client API, not Admin API.
- Admin Web remains the operator console.
- Management Context Links are permission-gated route links, not hidden shared
  privileged state.
- Tauri desktop playback should be Media Web-centered and should not assume
  WebView playback is sufficient for high-quality local media.
- Mobile clients remain native playback surfaces and should not inherit server
  administration workflows.

## Follow-On Candidates

- `identity-and-library-access-contract` (complete)
- `media-web-client-foundation` (active)
- `admin-media-management-context-links`
- `desktop-tauri-native-playback-spike`

## Resume Notes

Before implementation, re-read:

- `DESIGN.md`
- ADR 0024, 0026, 0027, and 0028
- `docs/workstreams/admin-web-v2-users-access-readiness/DESIGN.md`
- `docs/workstreams/android-client-foundation/UX_CONTEXT.md`
- `docs/workstreams/user-playback-state-contract/CONTRACT.md`
