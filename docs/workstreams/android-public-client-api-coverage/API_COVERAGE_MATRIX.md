# Android Public Client API Coverage Matrix

Status: Active
Last updated: 2026-05-19

Legend:

- `covered`: Android has a real production path that calls or builds the route.
- `partial`: Android receives related data, but the route is not first-class.
- `gap`: Public Client API route exists and Android should likely consume it.
- `defer`: Route is public but not needed for the current Android product slice.
- `non-goal`: Route is outside the phone/tablet client boundary.

## Public Client API V1 Routes

| Route | Android status | Current Android owner | Product decision |
| --- | --- | --- | --- |
| `GET /health` | covered | `TaruConnectionClient.testConnection` | Keep as setup preflight and version probe. |
| `GET /libraries?limit=&offset=` | covered | `TaruConnectionClient` auth probe; `TaruBrowseClient.listLibraries` | Keep. |
| `GET /libraries/{library_id}` | covered | `TaruBrowseClient.libraryDetail`; `LibraryDetailRouteContent` | Keep as the Library Detail summary route. |
| `GET /libraries/{library_id}/sources?limit=&offset=` | covered | `TaruBrowseClient.librarySources`; `LibraryDetailRouteContent` | Keep as safe source inventory, not as a fake poster grid. |
| `GET /items?limit=&offset=` | covered | `TaruBrowseClient.listItems` | Keep as Home/Libraries feed input. |
| `GET /items/{item_id}` | covered | `TaruBrowseClient.itemDetail` | Keep as primary detail aggregate. |
| `GET /items/{item_id}/credits` | partial | `GET /items/{item_id}` aggregate detail | Defer until credits need pagination or richer people context. |
| `GET /items/{item_id}/images` | covered | `TaruBrowseClient.itemImages`; Home/Libraries artwork enrichment | Keep as best-effort visible-page artwork enrichment. |
| `GET /images/{image_id}` | covered | `PublicArtworkSource`; `TaruArtworkImage` with Coil | Keep as authenticated selected artwork byte route. |
| `HEAD /images/{image_id}` | defer | none | Coil/cache behavior is enough for APIC-020; add explicit preflight only if cache diagnostics or validation UX needs it. |
| `GET /people?limit=&offset=` | gap | none | Defer until Browse People index exists. |
| `GET /people/{person_id}` | gap | none | Add with Person Detail if productizes actor/director pages. |
| `GET /people/{person_id}/items?limit=&offset=` | covered | `TaruBrowseClient.listPersonItems` | Keep as current cast/crew facet result path. |
| `GET /tags?limit=&offset=` | gap | none | Defer until Browse Tags index exists. |
| `GET /tags/{tag_id}/items?limit=&offset=` | covered | `TaruBrowseClient.listTagItems` | Keep as current tag facet result path. |
| `GET /genres?limit=&offset=` | gap | none | Defer until Browse Genres index exists. |
| `GET /genres/{genre_id}/items?limit=&offset=` | covered | `TaruBrowseClient.listGenreItems` | Keep as current genre facet result path. |
| `GET /search?q=&facet=&limit=&offset=` | covered | `TaruBrowseClient.searchItems` | Keep; broaden only when advanced filters become product scope. |
| `GET /sources/{source_id}/probe` | partial | playback decision response includes optional probe | Add only if Source Picker needs a refreshable probe detail panel. |
| `GET /sources/{source_id}/playback/decision` | covered | `TaruPlaybackClient.getPlaybackDecision` | Keep as playback launch gate. |
| `GET /sources/{source_id}/stream` | covered | `TaruPlaybackClient.directPlaybackTarget`; Media3 route | Keep as direct play target. |
| `HEAD /sources/{source_id}/stream` | covered | `TaruPlaybackClient.headDirectPlaybackTarget` | Keep as request builder; productize preflight only if needed. |
| `GET /sources/{source_id}/stream/remux` | covered | `TaruPlaybackClient.remuxPlaybackTarget`; Media3 route | Keep as remux target. |
| `GET /sources/{source_id}/stream/hls/playlist.m3u8` | covered | `TaruPlaybackClient.hlsPlaylistTarget`; Media3 route | Keep as transcode/HLS target. |
| `GET /playback/sessions/{session_id}` | covered | `TaruPlaybackClient.getPlaybackSession` | Keep for public transcode session inspection. |
| `POST /playback/sessions/{session_id}/cancel` | covered | `TaruPlaybackClient.cancelPlaybackSession` | Keep for best-effort route-exit cancellation when session id is known. |
| `GET /playback/sessions/{session_id}/hls/segments/{segment_name}` | covered | `TaruPlaybackClient.hlsSegmentTarget`; HLS playlist uses server URLs | Keep as request builder and server-provided playlist path. |

## Current Server Routes Outside Android Scope

These routes appear in `docs/api/HTTP_API.md` current route inventory but should
not be consumed by the Android phone/tablet app in this lane:

- storage diagnostics: `GET /storage/backends`, `GET /admin/v1/storage/staging`;
- library jobs and NFO operations: scan, NFO import, NFO export;
- metadata diagnostics and maintenance: refresh, attempts, raw cache, providers,
  maintenance jobs/plans, raw cleanup;
- webhooks and event delivery;
- addon registration, addon token/grant administration, addon side effects;
- automation providers/jobs/artifacts;
- admin overview, catalog governance, jobs, events, system config;
- managed artwork admin operations under `/admin/v1/artwork/...`;
- admin playback sessions/runtime diagnostics.

Android may indirectly benefit from these server capabilities through public
client DTOs, but it must not become a server administration surface.

## Priority Order

1. Source probe detail if Source Picker needs explicit technical media facts
   outside playback decision.
2. People/genre/tag index and person detail pages if browsing beyond metadata
   chips becomes a product goal.
3. Server contract follow-on for User Playback State before claiming
   cross-device Continue Watching.

## Non-Negotiable Client Rules

- Every authenticated route must use the active server profile and token
  reference.
- Diagnostics and previews must redact bearer tokens.
- Public API version mismatch remains a hard client error.
- Android must not parse raw storage locators, local paths, managed artwork
  storage URIs, provider secrets, or admin diagnostics.
- Device-local resume remains clearly separate from server-authoritative User
  Playback State.
