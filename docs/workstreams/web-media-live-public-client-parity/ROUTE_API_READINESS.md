# Web Media Live Public Client Parity - Route/API Readiness

Status: Active
Last updated: 2026-05-28

## WMLP-020 Audit Summary

The generated TypeScript SDK is sufficient for a first live Media read slice:
`listItems`, `searchItems`, `getItem`, item credits/images, playback decision,
browser playback ticket, playback sessions, HLS session segments, and user
playback-state reads/writes are exposed.

The main verified gap is library-scoped item browse. The SDK exposes
`listLibraries`, `getLibrary`, and `listLibrarySources`, but it does not expose a
`/libraries/{library_id}/items` route or a `library_id` filter on `listItems`.
`/media/library` must therefore show a truthful readiness state or an explicitly
named all-library fallback until a Public Client contract exists.

This audit verified the committed generated SDK in `sdk/typescript/src/index.ts`
and the committed route inventory from `crates/nako-client-protocol/src/lib.rs`
because the local protocol/API worktree has unrelated playback/subtitle and SDK
changes.

## Route Matrix

| Web route / surface | Current new `web/` state | Public Client readiness | Reentry decision |
| --- | --- | --- | --- |
| `/media` home rails | Fixture/live list seam exists through `listItems`; rails still use broad local categories. | Ready for first slice with `NakoClient.listItems({ limit, offset })`. `listContinueWatching` is available but should be wired after playback identity is stable. Dedicated "Recently Added" sort/filter is not proven. | WMLP-030 should wire live all-items rails with explicit empty/error states. Recently Added must be labeled as a contract gap unless a stable API sort/filter is added. |
| `/media/search` | Route owns `q`; current imported UI still mixes local/remote/download mock concepts. | Ready for local catalog search with `NakoClient.searchItems({ q, facet, limit, offset })`. Provider/indexer/acquisition search is not part of this lane. | WMLP-030 should replace mock local-result search with the Public Client search read model and keep acquisition/download surfaces out of runtime. |
| `/media/detail` | `web/src/api/public/media-data-source.ts` already calls `getItem`, but the route component still renders static fixture detail data. | Ready for detail read model with `NakoClient.getItem`, plus `listItemCredits`, `listItemImages`, and `managementContextLinks({ item_id, source_id, library_id })` if needed. `ItemDetailResponse.sources` provides source IDs for WMLP-040 playback entry. | WMLP-030 should introduce an explicit detail payload that carries item, images, sources, and missing-field states instead of adapting live data into the old fixture-only shape. |
| `/media/library` | Route owns id/view/sort/filter, but the component uses generated mock libraries and mock items. | Partially ready: `listLibraries`, `getLibrary`, and `listLibrarySources` exist. Not ready for item browse: no `listItems({ library_id })` query and no `/libraries/{library_id}/items` route are exposed in the SDK/route inventory. | WMLP-030 must not fake library-scoped live browse. It may show library metadata and source counts, or an explicitly named all-library fallback, while recording the missing browse contract. |
| Player entry | `VideoPlayer` can render browser-ticket media URLs and subtitle tracks, with fixture fallback when no live plan is available. | Browser ticket path is ready through `getPlaybackDecision`, `getSourceProbe`, and `createBrowserPlaybackTicket`. Session heartbeat is not ready because `BrowserPlaybackTicketResponse` does not expose a playback session id to the web client. | WMLP-040 wires browser-ticket playback and records heartbeat/session identity as a follow-on Public Client contract. |
| Continue watching | Local fixture cards exist. | Ready after WMLP-040: `listContinueWatching`, `getUserPlaybackState`, `updateUserPlaybackProgress`, and `setUserWatchedState` exist. DTOs include item, images, resume position, duration, progress, watched state, timestamps, and version. | WMLP-050 should wire continue-watching and progress/watched writes once playback session identity is stable. |
| Tauri desktop | Tauri build passes; playback remains browser-path only. | Browser Public Client route set is enough for WebView playback. Native desktop playback capability selection remains outside this lane. | WMLP closeout verifies Tauri build but does not implement native playback. |

## First Implementation Slice

WMLP-030 should implement the first slice in this order:

1. Add `web/src/api/public` read models for list/search/detail/library
   readiness, retaining fixture mode as an explicit mode.
2. Route `/media`, `/media/search`, and `/media/detail` through those read
   models with visible empty/error/readiness states.
3. Keep `/media/library` truthful: show library metadata/source readiness and do
   not claim library-scoped item browse until a Public Client contract exists.
4. Add route/data-source tests that lock live URLs, auth headers, fallback
   behavior, and missing-contract states.

## Known Follow-Ons

- Library-scoped Public Client item browse contract.
- Stable catalog sort/filter contract for Recently Added and watched filters.
- Browser playback ticket session-id exposure for web heartbeat.
- Desktop native playback capability matrix.
- Management Context Links.
- Invitations/account onboarding.
- Local-media recommendations.
- User playlists backend contract.
