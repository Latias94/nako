# Client Surface And Access Product Architecture - Milestones

Status: Closed
Last updated: 2026-06-01

## M0 - Product Architecture Accepted

Exit criteria:

- Admin Web, Media Web, desktop, and mobile surface boundaries are documented.
- Account/access staging is documented beyond Single-Admin Mode.
- Management Context Links are defined as a first-class UX pattern.
- Tauri desktop direction distinguishes WebView convenience from native
  playback quality.

## M1 - Identity Follow-On Split

Status: Complete

Exit criteria:

- A narrower identity/access workstream or ADR owns local accounts, roles,
  Library Access persistence, login/session behavior, and registration policy.
- It explicitly states that public registration is disabled by default unless
  a later operator opt-in model is implemented.
- Admin Web account CRUD remains blocked until backend authority exists.

## M2 - Media Web Follow-On Split

Status: Complete

Exit criteria:

- A Media Web foundation lane owns the first local-media browse/play routes.
- Public Client API gaps are listed before UI code depends on them.
- No Admin API DTOs are accepted as Media Web state.

## M3 - Context Switching Follow-On Split

Status: Complete

Exit criteria:

- DONE. A route/link matrix defines first Management Context Links from Media Web to
  Admin Web and reciprocal Admin links to Media Web.
- DONE. Role gating, Library Access, safe IDs, redaction, and confirmation
  ownership are part of the acceptance criteria.

Evidence: `docs/workstreams/admin-media-management-context-links/`.

## M4 - Desktop Spike Split Or Deferred

Status: Complete

Exit criteria:

- A desktop playback spike is opened, or desktop is explicitly deferred with
  the rationale recorded.
- The spike compares browser/WebView playback with native playback core
  integration, instead of assuming Tauri WebView is enough.

Decision: deferred from the MVP/browser-first path. Open a focused
`desktop-tauri-native-playback-spike` follow-on when desktop playback becomes a
product priority.

## M5 - Closeout

Exit criteria:

- This planning lane has no unowned product decisions blocking implementation.
- Follow-on lanes have authoritative docs and first executable tasks.
- Remaining risks are recorded in HANDOFF.md.

Status: Complete. CSAPA is closed after identity/access, Media Web, and
Management Context Links were split, and desktop playback was deferred.
