# Android Server-Backed Demo Fixtures — Route Matrix

Status: Closed
Last updated: 2026-05-18

## Decision

Use a real local `taru-server` seeded fixture as the first implementation
strategy for `profile-with-media`.

Reasoning:

- It exercises the real Public Client API, route middleware, DTO mapping, and
  playback selection path.
- The Android foundation walkthrough already proved that an emulator can reach
  a local Taru server through `adb reverse`.
- It avoids Android-only fake media data and avoids introducing a second mock
  server contract that could drift from Taru.
- A public-route-compatible test-server harness remains a fallback only if
  seeded server startup proves too slow, flaky, or hard to run on contributor
  machines.

The first fixture should be small: one Movies Media Library, one or two Media
Items, stable Genre/Tag/Person links, and one player-safe Media Source.

## Fixture Startup Shape

Preferred local shape for `ASD-030`:

1. Create a temporary fixture directory under an ignored path.
2. Write a Taru config with:
   - a loopback listen address;
   - one local Movies Media Library;
   - auth disabled or a documented harmless smoke token;
   - staging/cache roots inside the fixture directory.
3. Create a tiny valid video file and sidecar metadata for a deterministic
   Media Item.
4. Run existing Taru commands to scan and import metadata.
5. Start `taru-server serve`.
6. Run `adb reverse tcp:<port> tcp:<port>` so Android can use
   `http://127.0.0.1:<port>`.
7. Seed Android with one Server Profile that has a non-empty smoke token value
   because Android browse/playback clients require a non-blank access token.

Do not write raw token values, local filesystem paths, FFmpeg command lines, or
provider payloads into committed fixture reports.

## Android Surface Matrix

| Surface | Android client call | Public Client API route | Required fixture data | Status |
| --- | --- | --- | --- | --- |
| Connection preflight | `TaruConnectionClient` | `GET /health` and `GET /libraries?limit=1&offset=0` | Healthy server, Public Client API version header, at least one visible Media Library for the auth probe | Existing Android support; fixture must provide server |
| Home hero and library rows | `TaruBrowseClient.listLibraries`, `TaruBrowseClient.listItems` | `GET /libraries?limit=50&offset=0`, `GET /items?limit=24&offset=0` | One Movies Media Library, at least one Media Item with title, kind, overview/runtime/date if available | Existing Android support |
| Libraries tab | Same Home browse state | `GET /libraries`, `GET /items` | Same as Home; library options should include domain/preset where possible | Existing Android support |
| Search tab | `TaruBrowseClient.searchItems` | `GET /search?q=...&facet=...&limit=...&offset=...` | Search index entry for the demo title and optional facet | Existing Android support; optional for first smoke |
| Media Item detail | `TaruBrowseClient.itemDetail` | `GET /items/{item_id}` | Item detail with sources, genres, tags, credits, optional images, optional collections/studios | Existing Android support |
| Genre facet | `TaruBrowseClient.listGenreItems` | `GET /genres/{genre_id}/items?limit=24&offset=0` | Detail response must include `genres[].genre_id`; related item route returns the demo item | Existing Android support |
| Tag facet | `TaruBrowseClient.listTagItems` | `GET /tags/{tag_id}/items?limit=24&offset=0` | Detail response must include `tags[].tag_id`; related item route returns the demo item | Existing Android support |
| Person facet | `TaruBrowseClient.listPersonItems` | `GET /people/{person_id}/items?limit=24&offset=0` | Detail response must include `credits[].person_id`; related item route returns the demo item | Existing Android support, but rich credit names are still an API/UI gap |
| Source picker | Detail response plus `TaruPlaybackClient.getPlaybackDecision` | `GET /items/{item_id}`, `GET /sources/{source_id}/playback/decision?...` | At least one Media Source, optional probe, playback decision with `direct_play`, `remux`, or `transcode` plan | Existing Android support |
| Player-safe launch | `TaruPlaybackClient.recommendedPlaybackTarget` and Media3 route | `GET /sources/{source_id}/stream`, `GET /sources/{source_id}/stream/remux`, or `GET /sources/{source_id}/stream/hls/playlist.m3u8` | A target route that can open player UI without leaking paths or tokens; full streaming quality is not required for the first smoke | Fixture provider exists; needs smoke navigation |
| Playback session cancellation | Player dispose path | `POST /playback/sessions/{session_id}/cancel` | Only needed if fixture returns an HLS/remux session id and Android starts playback long enough to own it | Out of first smoke unless full stream validation is accepted |

## API Gaps To Keep Explicit

- Library-scoped item browsing is not currently a public-route-backed Android
  facet.
- Studio, Collection, Year, Media Item kind, and Source Mode facets are
  intentionally shown as API-gap routes in Android.
- `ItemDetailResponse.credits` gives relation ids and roles, but Android does
  not yet receive rich person display names in that same response.
- Source probe can be fetched through `GET /sources/{source_id}/probe`, but
  Android source picker currently relies on the optional `probe` included in
  `PlaybackDecisionResponse`.
- Android's stale `ClientTranscodePlan.inputLocator` requirement was removed in
  `ASD-030`; HLS/remux decisions can now decode without exposing
  `input_locator`.
- Full playback quality validation belongs to a later playback lane; this lane
  needs player-safe launch evidence first.

## Minimum Demo Data

Use stable, client-safe names:

- Media Library: `Movies`
- Media Item: `Night Harbor`
- Genre: `Mystery`
- Tag: `Lighthouse`
- Person: `Mira Vale`
- Media Source: `Night Harbor.mp4` for direct play where possible, or
  `Night Harbor.mkv` if the intended first decision is HLS/remux.

Prefer `mp4` direct play for the first player smoke because it reduces
transcode dependence. Keep an HLS/remux variant as a later extension after the
fixture startup command and smoke navigation are stable.

## Validation For This Discovery Slice

This route matrix was checked against:

- Android client calls in `apps/android/app/src/main/java/dev/taru/android/browse/TaruBrowseClient.kt`;
- Android playback calls in `apps/android/app/src/main/java/dev/taru/android/playback/TaruPlaybackClient.kt`;
- Android route state in `apps/android/app/src/main/java/dev/taru/android/ui/browse/TaruBrowseShell.kt`;
- Public OpenAPI route inventory in `crates/taru-api/src/openapi.rs`;
- protocol DTO source in `crates/taru-client-protocol/src/catalog.rs`;
- server HTTP routes in `crates/taru-server/src/http/catalog.rs`,
  `crates/taru-server/src/http/library.rs`, and
  `crates/taru-server/src/http/playback.rs`;
- previous real-server Android walkthrough evidence in
  `docs/workstreams/android-client-foundation/EVIDENCE_AND_GATES.md`.
