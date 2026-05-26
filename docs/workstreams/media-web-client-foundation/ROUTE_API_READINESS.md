# Media Web Route/API Readiness

Status: Draft
Last updated: 2026-05-26

This note records MWF-020 findings after checking the generated Public Client
OpenAPI and TypeScript SDK.

Validation run:

- `cargo test -p nako-api public_openapi -- --nocapture`
- `cargo run -q -p nako-api --example emit-typescript-sdk -- --output sdk/typescript/src/index.ts`

The generated SDK was already current after regeneration.

## Route Matrix

| Media Web route/workflow | Public Client support | Readiness | Notes |
| --- | --- | --- | --- |
| `/connect` server reachability | `health()` / `GET /health` | Ready | Health is unauthenticated. Authenticated current-principal/profile is not available. First UI can accept server URL and bearer token without inventing profile state. |
| Current account display | None | Gap | No Public Client `GET /users/me`, session, or profile route exists. Account switching can only mean clearing/replacing the active connection until credential/session work exists. |
| `/libraries` | `listLibraries()` / `GET /libraries` | Ready | Server now filters returned libraries by effective Library Access. |
| `/libraries/:libraryId` header | `getLibrary()` / `GET /libraries/{library_id}` | Ready | Server gates by browse Library Access. |
| `/libraries/:libraryId` source evidence | `listLibrarySources()` / `GET /libraries/{library_id}/sources` | Ready for MWF-040 | MWF-040 may show this as source evidence. It must not be labeled as the consumer library item grid. |
| `/libraries/:libraryId` item grid | Partial | Gap | `listLibrarySources()` exists, but it is source-centric. `listItems()` has only page query and cannot filter by library or sort by library item state. Add `GET /libraries/{library_id}/items` or a typed `library_id` filter on `GET /items` before building a real library browse grid. |
| Home Continue Watching | `listContinueWatching()` / `GET /users/me/playback-state/continue-watching` | Ready | Server filters returned items by current Library Access. |
| Home Recently Added | Partial | Gap | `listItems()` has no sort key or library/date-added query. Do not label a generic item page as Recently Added until a typed sort/filter contract exists. |
| `/search` | `searchItems()` / `GET /search` | Ready for MVP | Search supports `q`, `facet`, limit, and offset. Facets are lightweight strings without typed counts; acceptable for the first slice. |
| `/items/:itemId` | `getItem()`, `listItemCredits()`, `listItemImages()` | Ready | Item detail includes sources, credits, tags, genres, collections, studios, and selected artwork refs. |
| Artwork | `image()` / `GET /images/{image_id}` | Ready | Variant query uses safe selected artwork ids and dimensions. |
| Source/Version Picker | `ItemDetailResponse.sources`, `getSourceProbe()` | Ready for MVP | Enough for a source list using file names and technical facts. Rich Source Variant labels remain future work. |
| Playback decision | `getPlaybackDecision()` | Ready | Decision returns direct/remux/transcode mode without leaking server internals. |
| Browser direct/remux stream | `streamSource()`, `remuxStreamSource()` | Gap for native video element | SDK can fetch with bearer headers, but a normal `<video src>` cannot attach Authorization headers. Direct streaming through `fetch` is not a robust range-streaming player path. |
| Browser HLS | `hlsPlaylist()`, `hlsSegment()` | Partial | JavaScript HLS clients may attach headers. Native HLS playback cannot reliably attach bearer headers. The first player must choose a supported auth transport instead of assuming plain media URLs work. |
| Playback session status/cancel | `getPlaybackSession()`, `cancelPlaybackSession()` | Ready | Useful for playback UI and later diagnostics; admin diagnostics remain Admin Web-owned. |
| User Playback State | `getUserPlaybackState()`, `updateUserPlaybackProgress()`, `setUserWatchedState()` | Ready | Uses `/users/me` and hides principal identifiers. |

## Gaps

### MWF-GAP-001 - Current Principal Or Session Summary

Media Web has no Public Client route for "who am I?" or a session summary. For
MWF-030, the UI can show connection status based on a successful authenticated
Public Client call, but it must not invent display names, Role labels, or
account switching semantics.

Recommended follow-on:

- Add a small Public Client session/principal summary only after credential or
  session UX is accepted, or keep Media Web token-based for the first app
  scaffold.

### MWF-GAP-002 - Library-Scoped Item Browse

Media Web needs a user-facing library page. The current public route for a
library's contents is `GET /libraries/{library_id}/sources`, which is useful for
operator/source evidence but not ideal as the consumer item grid.

Recommended follow-on before replacing the source evidence list with a real
library item grid:

- Add `GET /libraries/{library_id}/items` with page, optional kind/facet/sort
  query, and server-side Library Access enforcement; or
- Extend `GET /items` with a typed `library_id` filter and explicit sort keys.

Do not use Admin API catalog governance routes for this.

MWF-040 intentionally uses `listLibrarySources()` as a source evidence list and
keeps this gap open for a later user-facing library item grid.

### MWF-GAP-003 - Recently Added Sort/Feed

The first home screen should avoid a fake Recently Added rail unless Public
Client API exposes an explicit sort/feed. Continue Watching is ready; Recently
Added can be deferred or implemented with a typed public sort key.

### MWF-GAP-004 - Browser Playback Auth Transport

The generated SDK can fetch stream responses with bearer headers. A browser
`<video>` element cannot attach those headers to `src` requests, and native HLS
has the same limitation on some platforms.

Recommended options before MWF-050:

- use a JavaScript HLS/MSE player path that can attach headers and document the
  browser support limits;
- add cookie/session auth for browser playback as part of credential/session
  work;
- add short-lived playback ticket URLs that preserve Library Access and
  playback-session policy without exposing privileged permanent URLs.

The first Media Web player should not pretend direct `<video src>` playback is
securely available with bearer-only auth.

## Accepted MWF-030 Scaffold Boundary

MWF-030 may still scaffold the Media surface inside `apps/admin-web` if it
stays within this boundary:

- connect to a server URL and bearer token;
- use `health()`, `listLibraries()`, `listItems()`, `searchItems()`, and item
  detail routes only where the SDK already has real contracts;
- fixture mode must be visibly development/test-only;
- do not implement a real library item grid, Recently Added rail, or browser
  player until the relevant gaps are resolved or consciously narrowed.
