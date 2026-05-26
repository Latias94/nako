# Media Web Client Foundation

Status: Active
Last updated: 2026-05-26

## Why This Lane Exists

Nako has an administration-first Admin Web, a Public Client API, generated
client contracts, user-scoped playback state, and effective Library Access
enforcement. It still lacks a first-party browser client where a viewer can
browse local media and play it without entering the operator console.

Admin Web must remain a governance and operations surface. Media Web should be
the browser playback surface. They should coexist in one frontend project first
so login state, navigation, and Management Context Links feel coherent, while
code boundaries still prevent Admin API data from becoming viewer state.

## Relevant Authority

- `CONTEXT.md`
- `PRODUCT.md`
- `DESIGN.md`
- `docs/adr/0024-inbound-token-authentication-boundary.md`
- `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/adr/0028-user-playback-state-principal-and-public-contract.md`
- `docs/workstreams/client-surface-and-access-product-architecture/`
- `docs/workstreams/identity-and-library-access-contract/`
- `docs/workstreams/public-client-api/`
- `docs/workstreams/openapi-client-contract/`
- `docs/workstreams/sdk-client-scaffold/`
- `docs/workstreams/typescript-sdk-package/`
- `docs/workstreams/user-playback-state-contract/`
- `docs/workstreams/playback-transcode-ops-hardening/`
- `apps/admin-web/README.md`
- `sdk/typescript/src/index.ts`

## Problem

The current browser story is split across surfaces:

- Admin Web has governance-oriented `/catalog` and `/items/:itemId` routes, but
  those routes intentionally avoid watch-first playback behavior.
- Public Client API can list libraries, browse/search Media Items, resolve
  playback decisions, serve artwork, and record User Playback State, but no
  first-party web app consumes it as a viewer.
- Identity/access now exists, but Media Web still needs an explicit login or
  connect model that does not imply open registration, password flows, or
  account switching that the backend cannot support yet.
- Desktop Tauri work should reuse the Media Web playback experience, but the
  browser client must land first so desktop does not start from Admin routes.
- Management Context Links need a media surface to link from, but the first
  Media Web slice should not block on admin-to-media and media-to-admin deep
  linking.

## Target State

When this lane closes:

- `apps/admin-web` contains route-owned Admin and Media surfaces with explicit
  `/admin/*` and `/media/*` namespaces or an equivalent accepted namespace.
- It consumes the Public Client API or generated Public Client SDK only.
- It has a truthful connect/login boundary for the current access model.
- It renders local Media Libraries, Media Library detail, search, Media Item
  detail, Source/Version Picker, and Player routes.
- It uses server-returned data as the source of truth for Library Access.
- It can show Continue Watching and save playback progress through User
  Playback State routes when those routes are available.
- It has fallback fixtures only for development and tests, clearly separated
  from live Public Client API data.
- It has no Admin API DTOs, raw Source Locators, local filesystem paths,
  provider payloads, tokens, FFmpeg command lines, or admin-only diagnostics in
  normal viewer UI.
- It records the Public Client API gaps that block richer local-media UX.
- Management Context Links and Tauri/native playback are split follow-ons
  unless this lane explicitly proves a narrow first link or package.

## In Scope

- Media Web product route map and UX constraints.
- Media surface scaffold under `apps/admin-web`.
- Public Client API data-source boundary and generated SDK usage.
- Connect/login MVP based on the currently accepted auth mechanism.
- Libraries, Media Library detail, search, Media Item detail, source selection,
  player shell, playback decision integration, and playback progress writes.
- Responsive desktop/mobile browser layout for the web client.
- Development fixtures and tests that do not pretend to be server authority.
- Browser smoke evidence for the first useful local-media path.

## Out Of Scope

- Admin API reads or writes.
- Admin Web account CRUD or Library Access policy editing.
- Public self-registration.
- Password hashing, reset tokens, invitations, OAuth/OIDC, LDAP, or passkeys.
- Management Context Links implementation beyond documenting required hooks.
- Tauri packaging and native player integration.
- Mobile native client changes.
- Recommendations, online media aggregation, social features, comments, or
  streaming-storefront discovery.
- Copying Jellyfin, Plex, or other reference UI/source/assets.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Media Web should share the frontend project but keep a separate surface namespace and API boundary. | High | The product needs Jellyfin-like Admin/Media switching, while ADR 0027 keeps Admin API and Public Client API separate. | Split a package later if desktop/player dependencies become heavy or the boundary is repeatedly violated. |
| Public Client API is sufficient for a first browsing/playback slice. | Medium | Existing Public Client API and generated SDK cover libraries, catalog, item detail, playback decisions, images, and User Playback State. | Add a narrow Public Client API gap task before building UI around hidden Admin API data. |
| The first connect/login UX can use the accepted bearer-token model without adding password credentials. | Medium | Identity/access persistence exists, but credential/session UX is intentionally split. | Split a credential/session workstream before shipping user-facing username/password login. |
| Public registration should stay disabled. | High | Nako is self-hosted private media software, and identity/access closeout defers invitation onboarding. | Opening registration requires abuse controls, invite/email semantics, audit, and recovery work. |
| Browser playback is enough for a web foundation but not the final desktop target. | High | ADR 0026 and product docs keep native playback core as the serious desktop direction. | If WebView proves insufficient even for browser MVP, narrow playback to direct/remux/HLS capability evidence and split native sooner. |

## Architecture Direction

### Surface Boundary

Media Web is a Client Application surface inside the shared web frontend. It
consumes Public Client API contracts and must not import Admin Web route models
or Admin API clients.

```text
apps/admin-web
  src/app-shell
    shared connection/session and surface switcher
  src/surfaces/admin
    operator routes, Admin API data source
  src/surfaces/media
    viewer routes, Public Client API data source
  src/shared
    safe UI primitives, routing helpers, i18n, test helpers
```

Admin routes may link to Media routes, and Media routes may expose admin-only
Management Context Links later. Those links are route URLs with stable safe IDs,
not shared privileged state.

### First Route Map

The first route set should be small enough to validate end to end:

- `/media/connect`: server URL and token/session entry when no connection
  exists.
- `/media`: home with Continue Watching and Recently Added when backed by public
  data.
- `/media/libraries`: accessible Media Libraries.
- `/media/libraries/:libraryId`: library browse, facets, and sort controls.
- `/media/search`: global search through Public Client API.
- `/media/items/:itemId`: Media Item detail with artwork, metadata, user state,
  and playable sources.
- `/media/watch/:itemId`: Source/Version Picker plus browser player.

Admin routes should use an explicit `/admin/*` namespace over time. Existing
Admin Web V2 routes may be kept as compatibility redirects during migration.

Route names can change during implementation, but the app must preserve the
same product separation: browse and playback first, admin context later.

### Auth And Account UX

The first UI should be truthful about current backend capability:

- accept a server URL and access token or the equivalent configured auth
  mechanism;
- show the resolved current principal if a public/session route exists, or show
  a constrained "connected" state without inventing profile data;
- support account switching only as clearing/replacing the current connection
  until real session/credential APIs exist;
- avoid public registration, password reset, invite redemption, and profile
  editing until backend authority exists.

### Public Data Source

Media Web should have an explicit data-source boundary similar in spirit to
Admin Web, but pointed at public contracts:

- production: generated Public Client SDK or thin public adapter;
- tests: deterministic route-local fixtures;
- development fallback: clearly labeled fixture mode when no live server is
  configured.

Any missing route should be recorded as a Public Client API gap. Do not bridge
through Admin API for viewer state.

### Playback

The browser player should use Public Client API playback decisions and the
server's direct stream, remux, HLS, or transcode routes. It should save progress
through User Playback State routes and surface client-safe playback errors.
Until a secure browser playback auth transport is accepted, Media Web may ship
a watch shell that shows source selection, playback decisions, and personal
playback state without rendering a real media element or stream URL.

Do not promise hardware acceleration, HDR, audio device routing, advanced
subtitle handling, or broad codec support inside this web lane. Those belong to
the desktop native playback spike or later playback hardening.

### UX Direction

Media Web should feel distinct from Admin Web:

- artwork-led browse and playback views;
- dense enough for library scanning and filtering, but not operator-dashboard
  dense;
- clear source/version choice when more than one playable Media Source exists;
- no admin-only diagnostics in normal viewer routes;
- admin links only after Management Context Links are accepted and gated.

## Closeout Condition

This lane can close when:

- the shared frontend has a validated first local-media browse/play path under
  the Media surface;
- auth/connect behavior is truthful for the backend capability that exists;
- Public Client API gaps are recorded or resolved;
- no Admin API dependency exists in the Media Web state path;
- package-local check/test/build and browser smoke evidence are recorded;
- follow-ons for Management Context Links, credentials/invitations, desktop
  Tauri/native playback, and richer recommendations are split or deferred.
