# Web Media Live Public Client Parity - Handoff

Status: Completed
Last updated: 2026-05-28

## Current State

WMLP-020 verified the generated Public Client SDK and committed route inventory
for Media Web live parity. The SDK has enough contract surface for the first
read slice: `listItems`, `searchItems`, `getItem`, item credits/images,
management context links, playback decision, browser playback ticket, playback
sessions, HLS session segments, and user playback-state reads/writes.

The main readiness gap is library-scoped item browse. `listLibraries`,
`getLibrary`, and `listLibrarySources` exist, but there is no SDK method or
route inventory entry for `/libraries/{library_id}/items`, and `listItems` does
not accept a `library_id` filter. `/media/library` must therefore stay truthful:
library metadata/source readiness is OK, but scoped item browse needs an
explicit missing-contract state or clearly labeled all-library fallback.

## Active Task

- Task ID: WMLP-020
- Owner: Codex
- Status: DONE
- Validation: `npm --prefix web run test -- src/test/data-source-contracts.test.ts` passed with 13 tests; `npm --prefix web run check` passed.

## Completed Task

- Task ID: WMLP-030
- Owner: Codex
- Status: DONE
- Validation: `npm --prefix web run test` passed with 47 tests; `npm --prefix web run check` passed; `npm --prefix web run build:budget` passed.

Completed substeps:

- WMLP-030A added Public Media read-model boundaries and contract tests for
  detail sources/images and library metadata/source readiness. It also keeps
  library-scoped item browse as an explicit missing Public Client contract.
- WMLP-030B routed detail/search through Public Media read models.
- WMLP-030C routed library metadata/source readiness without fake scoped item
  browse.

## Completed Task

- Task ID: WMLP-040
- Owner: Codex
- Status: DONE_WITH_CONCERNS
- Validation: playback data-source tests, route/player test, no-token assertion,
  `npm --prefix web run build:budget`

Completed:

- Browser ticket and subtitle URLs are available and tested.
- `VideoPlayer` renders browser-ticket media URLs and native subtitle tracks.
- Media and subtitle URL tests assert bearer tokens are not embedded.

Concern:

- At WMLP closeout, `BrowserPlaybackTicketResponse` did not expose a playback
  session id, so browser heartbeat was split to a named follow-on. That
  follow-on later closed in
  `docs/workstreams/public-client-browser-playback-session-identity/`.

## Completed Task

- Task ID: WMLP-050
- Owner: Codex
- Status: DONE
- Validation: `npm --prefix web run test` passed with 49 tests; `npm --prefix web run check` passed; `npm --prefix web run build:budget` passed.

Completed:

- Public Media data source exposes continue-watching, playback state read,
  progress update, and watched/unwatched writes.
- Home continue-watching reads Public Client playback-state data with fixture
  fallback.

## Closeout

- Task ID: WMLP-060
- Owner: planner
- Status: DONE
- Validation: `npm --prefix web run test`; `npm --prefix web run check`;
  `npm --prefix web run build:budget`; `npm --prefix web run tauri -- build`;
  browser smoke for Media routes; JSON validation; `git diff --check`.

## Follow-Ons

- Browser playback ticket session identity is now resolved by
  `docs/workstreams/public-client-browser-playback-session-identity/`.
- Public Client needs library-scoped item browse before `/media/library` can show
  scoped live items.
- Public Client needs stable catalog sort/filter before Recently Added and
  watched filters can be fully live.
- Desktop native playback remains a separate Tauri/native capability lane.

## Return Point

Return to WDRP-030 for Admin Operations reentry, or open one of the follow-on
contract lanes above.
