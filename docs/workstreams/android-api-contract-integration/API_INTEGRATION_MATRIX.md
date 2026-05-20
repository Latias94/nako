# Android API Integration Matrix

Status: Active
Last updated: 2026-05-20

Legend:

- `connected`: Android production code calls or builds the route.
- `productized`: Android exposes the route through a user-facing route/screen.
- `partial`: Android consumes related data but not the route itself.
- `split`: first-class target for a named follow-on workstream.
- `defer`: public route exists, but it is not needed for the current Android
  phone/tablet product slice.
- `out-of-scope`: route is not a Public Client API route for Android.

## Public Client API V1

| Route | Android status | Owner | Decision |
| --- | --- | --- | --- |
| `GET /health` | productized | `TaruConnectionClient.testConnection` | Keep as setup preflight and version probe. |
| `GET /libraries?limit=&offset=` | productized | `TaruConnectionClient`, `TaruBrowseClient.listLibraries` | Keep for auth probe and Home/Libraries. |
| `GET /libraries/{library_id}` | productized | `TaruBrowseClient.libraryDetail` | Keep for Library Detail summary. |
| `GET /libraries/{library_id}/sources?limit=&offset=` | productized | `TaruBrowseClient.librarySources` | Keep for source inventory. |
| `GET /items?limit=&offset=` | productized | `TaruBrowseClient.listItems` | Keep for Home feed. |
| `GET /items/{item_id}` | productized | `TaruBrowseClient.itemDetail` | Keep as primary detail aggregate. |
| `GET /items/{item_id}/credits` | partial | item detail aggregate | Defer standalone pagination until detail needs richer credits. |
| `GET /items/{item_id}/images` | connected | `TaruBrowseClient.itemImages` | Keep as visible-page artwork enrichment. |
| `GET /images/{image_id}` | productized | `PublicArtworkSource`, Coil | Keep as authenticated selected artwork bytes. |
| `HEAD /images/{image_id}` | defer | none | Add only if artwork diagnostics or cache preflight needs it. |
| `GET /people?limit=&offset=` | defer | `docs/workstreams/android-relationship-indexes/` | Top-level People Index deferred; Person Detail remains the primary People path. |
| `GET /people/{person_id}` | productized | `TaruBrowseClient.personDetail`, `TaruRoute.PersonDetail` | Keep as Cast & Crew Person Detail. |
| `GET /people/{person_id}/items?limit=&offset=` | productized | `TaruBrowseClient.listPersonItems` | Keep and reuse for Person Detail related Media Items. |
| `GET /tags?limit=&offset=` | split | `docs/workstreams/android-relationship-indexes/` | Accepted after Genres as the second relationship index slice. |
| `GET /tags/{tag_id}/items?limit=&offset=` | productized | `TaruBrowseClient.listTagItems` | Keep for tag chips and future Tags index. |
| `GET /genres?limit=&offset=` | split | `docs/workstreams/android-relationship-indexes/` | Accepted as the first relationship index slice. |
| `GET /genres/{genre_id}/items?limit=&offset=` | productized | `TaruBrowseClient.listGenreItems` | Keep for genre chips and future Genres index. |
| `GET /search?q=&facet=&limit=&offset=` | productized | `TaruBrowseClient.searchItems` | Keep; advanced filters are later UX scope. |
| `GET /sources/{source_id}/probe` | productized | `TaruPlaybackClient.getSourceProbe` | Keep for source facts. |
| `GET /sources/{source_id}/playback/decision` | productized | `TaruPlaybackClient.getPlaybackDecision` | Keep as playback launch gate. |
| `GET /sources/{source_id}/stream` | productized | `TaruPlaybackClient.directPlaybackTarget` | Keep for Direct Play. |
| `HEAD /sources/{source_id}/stream` | connected | `TaruPlaybackClient.headDirectPlaybackTarget` | Keep as request builder. |
| `GET /sources/{source_id}/stream/remux` | productized | `TaruPlaybackClient.remuxPlaybackTarget` | Keep for remux playback. |
| `GET /sources/{source_id}/stream/hls/playlist.m3u8` | productized | `TaruPlaybackClient.hlsPlaylistTarget` | Keep for HLS playback. |
| `GET /playback/sessions/{session_id}` | productized | `TaruPlaybackClient.getPlaybackSession` | Keep for session inspection. |
| `POST /playback/sessions/{session_id}/cancel` | productized | `TaruPlaybackClient.cancelPlaybackSession` | Keep for route-exit cancellation. |
| `GET /playback/sessions/{session_id}/hls/segments/{segment_name}` | connected | `TaruPlaybackClient.hlsSegmentTarget` | Keep as request builder; playlist owns segment loading. |
| `GET /users/me/playback-state/items/{item_id}` | productized | `TaruUserPlaybackClient.getState` | Keep for detail resume and watched state. |
| `GET /users/me/playback-state/continue-watching?limit=&offset=` | productized | `TaruUserPlaybackClient.continueWatching` | Keep for Home Continue Watching. |
| `PUT /users/me/playback-state/items/{item_id}/progress` | productized | playback exit effects | Keep for playback progress reporting. |
| `PUT /users/me/playback-state/items/{item_id}/watched` | productized | playback exit effects | Keep for watched reporting. |

## Out Of Android Scope

Android phone/tablet must not call current server routes for storage
diagnostics, library scan/NFO jobs, metadata diagnostics, webhooks, Addons,
Automation, Admin overview/events/jobs, Admin artwork management, Admin
playback diagnostics, or system config. If a phone/tablet feature needs one of
those capabilities, first design a Public Client API route instead of consuming
the Admin/internal route directly.

## First Product Gap

The cleanest productization gap for this lane was Person Detail route state
and UI because:

- item detail already has Cast & Crew rows with stable person IDs;
- `GET /people/{person_id}` is connected through
  `TaruBrowseClient.personDetail` and productized as `TaruRoute.PersonDetail`;
- `GET /people/{person_id}/items` is already connected and smoke-covered as a
  facet route;
- Person Detail is a natural user workflow in Jellyfin/Plex-style browsing;
- it exercises a new route/state/UI shape without requiring broad indexes,
  advanced search filters, or server API changes.

People, Tags, and Genres index pages are split to
`docs/workstreams/android-relationship-indexes/` because their value depends on
browse information architecture, not on the Person Detail API contract proof.
