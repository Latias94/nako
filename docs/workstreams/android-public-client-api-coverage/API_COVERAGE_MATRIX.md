# Android Public Client API Coverage Matrix

Status: Closed
Last updated: 2026-05-20

Legend:

- `covered`: Android has a real production path that calls or builds the route.
- `partial`: Android receives related data, but the route is not first-class.
- `gap`: Public Client API route exists and Android should likely consume it.
- `defer`: Route is public but not needed for the current Android product slice.
- `non-goal`: Route is outside the phone/tablet client boundary.

## Public Client API V1 Routes

| Route | Android status | Current Android owner | Product decision |
| --- | --- | --- | --- |
| `GET /health` | covered | `NakoConnectionClient.testConnection` | Keep as setup preflight and version probe. |
| `GET /libraries?limit=&offset=` | covered | `NakoConnectionClient` auth probe; `NakoBrowseClient.listLibraries` | Keep. |
| `GET /libraries/{library_id}` | covered | `NakoBrowseClient.libraryDetail`; `LibraryDetailRouteContent` | Keep as the Library Detail summary route. |
| `GET /libraries/{library_id}/sources?limit=&offset=` | covered | `NakoBrowseClient.librarySources`; `LibraryDetailRouteContent` | Keep as safe source inventory, not as a fake poster grid. |
| `GET /items?limit=&offset=` | covered | `NakoBrowseClient.listItems` | Keep as Home/Libraries feed input. |
| `GET /items/{item_id}` | covered | `NakoBrowseClient.itemDetail` | Keep as primary detail aggregate. |
| `GET /items/{item_id}/credits` | partial | `GET /items/{item_id}` aggregate detail | Defer until credits need pagination or richer people context. |
| `GET /items/{item_id}/images` | covered | `NakoBrowseClient.itemImages`; Home/Libraries artwork enrichment | Keep as best-effort visible-page artwork enrichment. |
| `GET /images/{image_id}` | covered | `PublicArtworkSource`; `NakoArtworkImage` with Coil | Keep as authenticated selected artwork byte route. |
| `HEAD /images/{image_id}` | defer | none | Coil/cache behavior is enough for APIC-020; add explicit preflight only if cache diagnostics or validation UX needs it. |
| `GET /people?limit=&offset=` | defer | none | Defer until Browse People index exists. |
| `GET /people/{person_id}` | defer | none | Add with Person Detail if productizes actor/director pages. |
| `GET /people/{person_id}/items?limit=&offset=` | covered | `NakoBrowseClient.listPersonItems` | Keep as current cast/crew facet result path. |
| `GET /tags?limit=&offset=` | covered | `NakoBrowseClient.listTags`; `docs/workstreams/android-tags-index/` | Typed client contract exists; Tags Index route state remains in the follow-on lane. |
| `GET /tags/{tag_id}/items?limit=&offset=` | covered | `NakoBrowseClient.listTagItems` | Keep as current tag facet result path and future Tags Index destination. |
| `GET /genres?limit=&offset=` | covered | `NakoBrowseClient.listGenres`; `NakoRoute.RelationshipIndex(Genres)` | Browse Genres index is implemented and smoke-proven. |
| `GET /genres/{genre_id}/items?limit=&offset=` | covered | `NakoBrowseClient.listGenreItems` | Keep as current genre facet result path and Genres Index destination. |
| `GET /search?q=&facet=&limit=&offset=` | covered | `NakoBrowseClient.searchItems` | Keep; broaden only when advanced filters become product scope. |
| `GET /sources/{source_id}/probe` | covered | `NakoPlaybackClient.getSourceProbe`; `SourcePickerSurface` source facts | Keep as the Source Picker source-facts route, separate from playback decision. |
| `GET /sources/{source_id}/playback/decision` | covered | `NakoPlaybackClient.getPlaybackDecision` | Keep as playback launch gate. |
| `GET /sources/{source_id}/stream` | covered | `NakoPlaybackClient.directPlaybackTarget`; Media3 route | Keep as direct play target. |
| `HEAD /sources/{source_id}/stream` | covered | `NakoPlaybackClient.headDirectPlaybackTarget` | Keep as request builder; productize preflight only if needed. |
| `GET /sources/{source_id}/stream/remux` | covered | `NakoPlaybackClient.remuxPlaybackTarget`; Media3 route | Keep as remux target. |
| `GET /sources/{source_id}/stream/hls/playlist.m3u8` | covered | `NakoPlaybackClient.hlsPlaylistTarget`; Media3 route | Keep as transcode/HLS target. |
| `GET /playback/sessions/{session_id}` | covered | `NakoPlaybackClient.getPlaybackSession` | Keep for public transcode session inspection. |
| `POST /playback/sessions/{session_id}/cancel` | covered | `NakoPlaybackClient.cancelPlaybackSession` | Keep for best-effort route-exit cancellation when session id is known. |
| `GET /playback/sessions/{session_id}/hls/segments/{segment_name}` | covered | `NakoPlaybackClient.hlsSegmentTarget`; HLS playlist uses server URLs | Keep as request builder and server-provided playlist path. |

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

## Closed Follow-Ons

- People/genre/tag index and person detail pages if browsing beyond metadata
   chips becomes a product goal.
- Server-authoritative **User Playback State**:
  `docs/workstreams/user-playback-state-contract/`.

## Non-Negotiable Client Rules

- Every authenticated route must use the active server profile and token
  reference.
- Diagnostics and previews must redact bearer tokens.
- Public API version mismatch remains a hard client error.
- Android must not parse raw storage locators, local paths, managed artwork
  storage URIs, provider secrets, or admin diagnostics.
- Device-local resume remains clearly separate from server-authoritative User
  Playback State.
