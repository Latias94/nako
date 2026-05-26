# Admin Web V2 Library Management And Localization - Handoff

Status: Closed
Last updated: 2026-05-25

## Current State

The lane is closed. AWL-010 through AWL-060 are complete:

- `/libraries/:libraryId` is route-owned and reachable from `/libraries`.
- The detail route composes redacted system config diagnostics with the existing
  Admin metadata-profile read route.
- Source inventory uses an explicitly named public-read bridge summary; scan/NFO
  commands are user-triggered Admin API wrappers and do not fire on page load.
- Admin Web now has a small dependency-free i18n boundary with English and
  Simplified Chinese catalogs, a shell locale selector, localized SourceLabel
  text, and message-id coverage for the app shell plus library management
  routes.
- `/libraries/:libraryId` now exposes Metadata Profile as an explicit
  full-replacement GET/PUT workflow, summarizes Source inventory through the
  public-read bridge, and queues scan/NFO import/export only after user
  confirmation through Admin API command wrappers.
- AWL-050 is complete. `PARITY_GAP_SPLIT.md` re-scores remaining Admin Web V2
  parity gaps and recommends `admin-web-v2-media-browsing-and-item-detail-governance`
  as the next bounded execution lane.

## Active Task

- Task ID: none in this lane
- Owner: none
- Files: none
- Validation: complete; see `EVIDENCE_AND_GATES.md`
- Status: CLOSED
- Review: complete, no blocking findings
- Evidence: `CLOSEOUT.md` and `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- New workstream opened instead of reopening the closed read-only
  `admin-web-v2-media-libraries-route` lane.
- First implementation should be route-owned library detail, not a broad
  settings or catalog rewrite.
- Localization starts as infrastructure plus route-local migration, not a
  whole-app translation sweep.
- Admin Web may use public read routes for library/source inventory only through
  an explicitly named bridge; admin mutations stay Admin API owned.
- Source inventory was not wired in AWL-020. AWL-030 resolved it as an
  explicitly named public-read bridge summary instead of a raw public route dump.
- Locale catalogs translate product UI copy, not API enum values, library IDs,
  provider IDs, or diagnostic facts operators compare with backend output.
- Scan/NFO actions are Admin API owned for Admin Web. The legacy public
  mutation routes remain available for existing clients, but Admin Web now uses
  `/admin/v1/libraries/{library_id}/scan`,
  `/admin/v1/libraries/{library_id}/nfo/import`, and
  `/admin/v1/libraries/{library_id}/nfo/export`.
- Metadata Profile editing deliberately uses full PUT replacement. If that
  proves too blunt operationally, split field-specific patching into a follow-on
  instead of hiding the authority model in the UI.
- Remaining Jellyfin/Plex-style management parity should not stay in this lane.
  The next recommended lane is media browsing and item detail governance; other
  splits are settings/network mutation authority, users/roles/Library Access,
  governance repair actions, Addon operations mutations, playback support
  detail, and route-by-route i18n expansion.

## Blockers

- None for this closed lane.

## Next Recommended Action

Open `admin-web-v2-media-browsing-and-item-detail-governance` before
implementing media browsing or item detail work. Keep settings mutation,
users/Library Access, governance repair actions, Addon operation mutations,
playback support detail, and broad i18n expansion in separate follow-on lanes.
