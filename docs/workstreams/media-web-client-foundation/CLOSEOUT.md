# Media Web Client Foundation Closeout

Status: Closed
Closed: 2026-05-26

## Closeout Claim

This lane is complete for the Media Web foundation target. `apps/admin-web` now
contains a first-party Media surface beside Admin Web, with explicit route
boundaries, Public Client SDK data-source usage, local-media browse/search,
Media Item detail, Source/Version Picker, a safe watch shell, and User Playback
State reads/writes.

The lane does not claim a real browser video player. Browser stream playback
remains split to the playback auth transport follow-on because bearer-only
`<video src>` is not a secure or correct contract.

## Delivered

- Shared Admin/Media frontend shell inside `apps/admin-web`.
- Media connect MVP with fixture/live Public Client SDK data-source boundary.
- Media home with Continue Watching and Media Items from Public Client data.
- `/media/libraries`, `/media/libraries/:libraryId`, `/media/search`, and
  `/media/items/:itemId`.
- URL-owned pagination/search/source selection state.
- `/media/watch/:itemId` safe watch shell with Source/Version Picker and
  playback decision preview.
- User Playback State read plus watched/unwatched writes through `/users/me`.
- Redaction tests and boundary checks for Admin API, raw Source Locators, local
  paths, ffmpeg details, bearer tokens, and privileged stream URLs.
- Desktop and mobile browser smoke evidence for fixture routes.
- `FOLLOW_ON_SPLIT.md` with bounded next lanes.

## Review Result

### Workstream Compliance

- Blocking: none.
- Important: none.
- `TODO.md` tasks MWF-010 through MWF-060 are complete.
- `DESIGN.md` target is satisfied for the foundation scope, with real browser
  playback explicitly split.
- ADR 0027 is respected: Media Web consumes Public Client API contracts and
  does not import Admin API state.
- ADR 0028 is respected: playback state is current-principal scoped through
  `/users/me`.

### Code Quality

- Blocking: none.
- Important: none.
- Media data access is centralized behind `MediaWebDataSource`.
- Tests exercise route behavior through App/UI/data-source seams rather than
  implementation internals.
- Source selection is URL-owned and accessible through `aria-pressed`.
- The watch shell does not render a fake media element or mint a stream URL.

### Missing Gates

- None for the shipped frontend/docs scope.
- Rust/Public API gates were not rerun during MWF-060 because MWF-050/MWF-060
  changed only `apps/admin-web` and workstream docs. Public Client OpenAPI and
  SDK readiness were verified in MWF-020 and no Rust/API/SDK files changed in
  this closeout slice.

## Follow-Ons

See `FOLLOW_ON_SPLIT.md`.

Recommended next lane:

1. Browser Playback Auth Transport

Additional bounded lanes:

2. Credential And Session UX
3. Management Context Links
4. Library-Scoped Browse And Recently Added
5. Desktop Tauri Native Playback
6. Invitation Onboarding
7. Local-Media Recommendations

## Evidence Anchors

- `docs/workstreams/media-web-client-foundation/EVIDENCE_AND_GATES.md`
- `docs/workstreams/media-web-client-foundation/FOLLOW_ON_SPLIT.md`
- `docs/workstreams/media-web-client-foundation/ROUTE_API_READINESS.md`
- `apps/admin-web/src/surfaces/media/MediaPages.tsx`
- `apps/admin-web/src/surfaces/media/mediaDataSource.ts`
- `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx`
- `apps/admin-web/src/surfaces/media/mediaDataSource.test.ts`
