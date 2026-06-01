# Client Surface And Access Product Architecture - Handoff

Status: Closed
Last updated: 2026-06-01

## Current State

This planning lane is closed. CSAPA-010 drafted the product architecture for
Admin Web, Media Web, desktop, mobile, accounts, roles, Library Access, and
Management Context Links.

CSAPA-020 is complete through `identity-and-library-access-contract`. The
backend now has durable identity/access persistence, bootstrap administrator
semantics, Admin API access-management routes, and Public Client API
effective-access enforcement.

CSAPA-030 is complete through `media-web-client-foundation`. The Media Web
foundation lane is closed; later product frontend work now targets `web/`.

CSAPA-040 is complete through `admin-media-management-context-links`. Backend
Management Context Links already exist at `/management/context-links`; the new
frontend lane owns `web/` consumption, route resolution, and cross-surface
verification.

## Closed State

CSAPA-050 explicitly deferred desktop playback strategy from the MVP/browser-first
path. A future desktop effort should open a focused
`desktop-tauri-native-playback-spike` workstream with platform playback evidence.

CSAPA-060 closed this broad product architecture lane. Future implementation
must open narrower follow-ons.

## Key Constraints

- Media Web must consume Public Client API, not Admin API.
- Admin Web remains the operator console.
- Management Context Links are backend-computed, permission-gated route links,
  not hidden shared privileged state.
- Tauri desktop playback should be Media Web-centered and should not assume
  WebView playback is sufficient for high-quality local media.
- Mobile clients remain native playback surfaces and should not inherit server
  administration workflows.

## Follow-On Candidates

- `identity-and-library-access-contract` (complete)
- `media-web-client-foundation` (closed)
- `admin-media-management-context-links` (active)
- `desktop-tauri-native-playback-spike` (deferred/proposed)

## Resume Notes

Before future implementation, re-read:

- `DESIGN.md`
- ADR 0024, 0026, 0027, and 0028
- `docs/workstreams/admin-web-v2-users-access-readiness/DESIGN.md`
- `docs/workstreams/android-client-foundation/UX_CONTEXT.md`
- `docs/workstreams/user-playback-state-contract/CONTRACT.md`
