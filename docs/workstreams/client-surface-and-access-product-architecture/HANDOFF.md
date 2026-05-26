# Client Surface And Access Product Architecture - Handoff

Status: Draft
Last updated: 2026-05-26

## Current State

This planning lane is open. CSAPA-010 drafted the product architecture for
Admin Web, Media Web, desktop, mobile, accounts, roles, Library Access, and
Management Context Links.

No code has been changed. No runtime behavior is claimed.

## Next Recommended Task

CSAPA-020: split or write the identity/access contract follow-on.

Recommended first decisions:

- keep Single-Admin Mode as the bootstrap mode;
- add local admin-created accounts before public registration;
- disable public registration by default;
- model coarse roles plus Library Access before fine-grained permissions;
- decide whether first login uses bearer tokens, username/password sessions,
  or an intermediate local account token model.

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

- `identity-and-library-access-contract`
- `media-web-client-foundation`
- `admin-media-management-context-links`
- `desktop-tauri-native-playback-spike`

## Resume Notes

Before implementation, re-read:

- `DESIGN.md`
- ADR 0024, 0026, 0027, and 0028
- `docs/workstreams/admin-web-v2-users-access-readiness/DESIGN.md`
- `docs/workstreams/android-client-foundation/UX_CONTEXT.md`
- `docs/workstreams/user-playback-state-contract/CONTRACT.md`
