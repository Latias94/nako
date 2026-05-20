# Android API Contract Integration

Status: Closed
Last updated: 2026-05-20

## Problem

Android already calls the server-backed Public Client API for connection
checks, home feed, library detail, item detail, search, selected artwork,
source probe, playback decision, stream targets, playback sessions, and
server-authoritative User Playback State. The remaining risk is no longer
"Android is mostly mocked"; it is that public route coverage and product
navigation are tracked in several closed lanes, while the next user-facing
API slices are not owned by one current execution lane.

The most important unfinished product surface is relationship browsing:
People, Tags, Genres, and Person Detail. Item detail already exposes
relationship chips and Cast & Crew rows, and the API already exposes public
routes for related Media Items. Android needs a clean route/state/UI contract
for these surfaces instead of adding one-off screens or keeping
`apiGapState` placeholders forever.

## Target State

- `API_INTEGRATION_MATRIX.md` is the current Android-facing source of truth for
  Public Client API route status.
- Android keeps production code restricted to public client routes and does
  not consume admin/internal server APIs.
- Relationship browsing has explicit routes, state, data-source methods, and
  Material Expressive screens for the productized subset.
- Missing public routes are either implemented, intentionally deferred, or
  recorded as server/API follow-ons.
- Smoke and unit validation prove at least one real server-backed relationship
  browsing path end to end.

## Closeout

Closed on 2026-05-20 after Person Detail was connected, productized, and
smoke-proven. People, Tags, and Genres index pages are split to
`docs/workstreams/android-relationship-indexes/` because they are a separate
browse information architecture lane.

## Scope

- Android Public Client API integration docs and matrix.
- Android browse client DTOs and route methods for public relationship routes.
- Android browse session state/actions/navigation for relationship index and
  person detail routes.
- Compose screens for People/Tags/Genres indexes and Person Detail when they
  become task scope.
- Smoke fixture and smoke assertions for the first server-backed relationship
  browsing path.

## Non-Goals

- Do not consume Admin, Addon, Automation, metadata maintenance, storage
  diagnostics, or job control routes from Android.
- Do not rewrite the HTTP stack or replace the existing `TaruHttpTransport`
  seam.
- Do not add a generated Kotlin SDK in this lane unless the matrix shows the
  manual client is the blocking source of drift.
- Do not implement broad pagination, filtering, or offline caching until the
  first route/state/UI contract is proved.
- Do not change server API shapes unless Android discovers an actual public
  contract gap.

## Architecture Direction

Keep Android API code divided by user intent:

- `connection`: setup and safe server profile validation;
- `browse`: catalog, relationship, image-reference, and detail reads;
- `playback`: source probe, playback decision, stream targets, and session
  inspection/cancellation;
- `userplayback`: `/users/me` playback state and Continue Watching.

Relationship browsing should extend the existing browse route model instead of
creating detached screen-local network calls. Route state should remain
unidirectional: user intent dispatches a `BrowseAction`, `BrowseSession`
prepares route state, and `ClientBrowseDataSource` owns the public API calls.

## First Slice

APICI-010 creates the current integration matrix and task ledger. The first
implementation slice after APICI-010 should be a narrow Person Detail route:

- add `GET /people/{person_id}` client coverage;
- open a Person Detail route from Cast & Crew rows with stable IDs;
- show the person summary plus their related Media Items using existing
  `GET /people/{person_id}/items`;
- prove it with focused unit tests and the existing `profile-with-media`
  smoke fixture.
