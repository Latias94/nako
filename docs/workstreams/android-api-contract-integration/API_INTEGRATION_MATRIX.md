# Android API Integration Matrix

Status: Closed
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
| `GET /health` | productized | `NakoConnectionClient.testConnection` | Keep as setup preflight and version probe. |
| `GET /libraries?limit=&offset=` | productized | `NakoConnectionClient`, `NakoBrowseClient.listLibraries` | Keep for auth probe and Home/Libraries. |
| `GET /libraries/{library_id}` | productized | `NakoBrowseClient.libraryDetail` | Keep for Library Detail summary. |
| `GET /libraries/{library_id}/sources?limit=&offset=` | productized | `NakoBrowseClient.librarySources` | Keep for source inventory. |
| `GET /items?limit=&offset=` | productized | `NakoBrowseClient.listItems` | Keep for Home feed. |
| `GET /items/{item_id}` | productized | `NakoBrowseClient.itemDetail` | Keep as primary detail aggregate. |
| `GET /items/{item_id}/credits` | partial | item detail aggregate | Defer standalone pagination until detail needs richer credits. |
| `GET /items/{item_id}/images` | connected | `NakoBrowseClient.itemImages` | Keep as visible-page artwork enrichment. |
| `GET /images/{image_id}` | productized | `PublicArtworkSource`, Coil | Keep as authenticated selected artwork bytes. |
| `HEAD /images/{image_id}` | defer | none | Add only if artwork diagnostics or cache preflight needs it. |
| `GET /people?limit=&offset=` | defer | `docs/workstreams/android-relationship-indexes/` | Top-level People Index deferred; Person Detail remains the primary People path. |
| `GET /people/{person_id}` | productized | `NakoBrowseClient.personDetail`, `NakoRoute.PersonDetail` | Keep as Cast & Crew Person Detail. |
| `GET /people/{person_id}/items?limit=&offset=` | productized | `NakoBrowseClient.listPersonItems` | Keep and reuse for Person Detail related Media Items. |
| `GET /tags?limit=&offset=` | connected | `NakoBrowseClient.listTags`; `docs/workstreams/android-tags-index/` | Typed Android contract is in place; route state and screen remain active in the Tags Index lane. |
| `GET /tags/{tag_id}/items?limit=&offset=` | productized | `NakoBrowseClient.listTagItems` | Keep for tag chips and Tags Index. |
| `GET /genres?limit=&offset=` | productized | `NakoBrowseClient.listGenres`; `NakoRoute.RelationshipIndex(Genres)` | Genres Index is implemented and smoke-proven. |
| `GET /genres/{genre_id}/items?limit=&offset=` | productized | `NakoBrowseClient.listGenreItems` | Keep for genre chips and Genres Index rows. |
| `GET /search?q=&facet=&limit=&offset=` | productized | `NakoBrowseClient.searchItems` | Keep; advanced filters are later UX scope. |
| `GET /sources/{source_id}/probe` | productized | `NakoPlaybackClient.getSourceProbe` | Keep for source facts. |
| `GET /sources/{source_id}/playback/decision` | productized | `NakoPlaybackClient.getPlaybackDecision` | Keep as playback launch gate. |
| `GET /sources/{source_id}/stream` | productized | `NakoPlaybackClient.directPlaybackTarget` | Keep for Direct Play. |
| `HEAD /sources/{source_id}/stream` | connected | `NakoPlaybackClient.headDirectPlaybackTarget` | Keep as request builder. |
| `GET /sources/{source_id}/stream/remux` | productized | `NakoPlaybackClient.remuxPlaybackTarget` | Keep for remux playback. |
| `GET /sources/{source_id}/stream/hls/playlist.m3u8` | productized | `NakoPlaybackClient.hlsPlaylistTarget` | Keep for HLS playback. |
| `GET /playback/sessions/{session_id}` | productized | `NakoPlaybackClient.getPlaybackSession` | Keep for session inspection. |
| `POST /playback/sessions/{session_id}/cancel` | productized | `NakoPlaybackClient.cancelPlaybackSession` | Keep for route-exit cancellation. |
| `GET /playback/sessions/{session_id}/hls/segments/{segment_name}` | connected | `NakoPlaybackClient.hlsSegmentTarget` | Keep as request builder; playlist owns segment loading. |
| `GET /users/me/playback-state/items/{item_id}` | productized | `NakoUserPlaybackClient.getState` | Keep for detail resume and watched state. |
| `GET /users/me/playback-state/continue-watching?limit=&offset=` | productized | `NakoUserPlaybackClient.continueWatching` | Keep for Home Continue Watching. |
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
  `NakoBrowseClient.personDetail` and productized as `NakoRoute.PersonDetail`;
- `GET /people/{person_id}/items` is already connected and smoke-covered as a
  facet route;
- Person Detail is a natural user workflow in Jellyfin/Plex-style browsing;
- it exercises a new route/state/UI shape without requiring broad indexes,
  advanced search filters, or server API changes.

People, Tags, and Genres index pages were split to
`docs/workstreams/android-relationship-indexes/` because their value depends on
browse information architecture, not on the Person Detail API contract proof.
Genres Index is now complete there; Tags Index continues in
`docs/workstreams/android-tags-index/`.
