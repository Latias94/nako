# Android Material Expressive UI

Status: Completed
Last updated: 2026-05-18

## Why This Lane Exists

The Android Client Foundation intentionally proved connection, browse, search,
detail, playback decision, Media3 launch, and playback-session boundaries before
settling the production UI architecture. That left a useful but tracer-shaped
Compose app: dense enough to exercise the Public Client API, but not yet
immersive, cohesive, or clearly Material 3 Expressive.

This lane rewrites the Android UI layer around the approved V2 direction:
regular Compose-friendly structure, Material 3 as the interaction contract,
expressive media surfaces through artwork, restrained motion, dynamic color,
and clear playback/source choices. Compatibility with the old UI code is not a
goal; clean architecture and correct ownership are preferred.

## Relevant Authority

- `CONTEXT.md`
- `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
- `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md`
- `docs/adr/0025-openapi-public-client-sdk-contract.md`
- `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- `docs/workstreams/android-client-foundation/UX_CONTEXT.md`
- `docs/workstreams/android-client-foundation/CLIENT_INTERFACE_DESIGN.md`
- `docs/workstreams/android-client-foundation/reference-screenshots/`
- `docs/workstreams/public-client-api/`

## Problem

- Current UI was implemented before the interface direction was fully settled.
- Home and detail screens are structurally functional but still read like
  implementation tracers instead of a cinematic playback client.
- Material 3 is present, but expressive behavior is shallow: dynamic color,
  motion, state transitions, and navigation chrome are not yet productized.
- The existing `ui/browse` package mixes route orchestration, loading state,
  screen composition, reusable components, and placeholder behavior too tightly.
- Reference screenshots and V2 decisions exist, but there is no active
  execution ledger that turns them into Android implementation slices.

## Target State

- Android UI has a small, explicit design-system layer for color, typography,
  spacing, shape, motion, adaptive chrome, and artwork-derived accents.
- The app remains dark-first and playback-first, with Settings and setup kept
  restrained.
- Material 3 dynamic color is supported as a user/system option for app chrome,
  while media-artwork accents are local to Home, detail, source picker, and
  player surfaces.
- Motion is purposeful and bounded: route transitions, press/selection feedback,
  source selection, playback loading, and sheet reveal. Decorative page-load
  choreography is out.
- Top-level navigation uses bottom navigation on phones and a navigation rail
  on wider layouts without changing the product model.
- Home, Libraries, Search, Media Item Detail, Source / Version Picker, Player,
  and Settings share one component vocabulary and clear loading/empty/error
  states.
- The implementation may delete or replace old Compose surfaces when a cleaner
  boundary exists.

## In Scope

- Compose UI rewrite under `apps/android/app/src/main/java/dev/nako/android/ui`.
- Theme/token refactor for Material 3, dynamic color, dark-first roles, motion,
  and artwork accent hooks.
- Reusable media-client components: app chrome, poster/backdrop surfaces,
  metadata chips, section rails, source summary, action cluster, state cards,
  and settings rows.
- Phone and tablet adaptive layout for top-level chrome and detail surfaces.
- V2 reference-screen implementation for Home, Media Item Detail,
  Source / Version Picker, Browse Facet Result, Settings Home, and Server
  Profile.
- Preservation of existing Public Client API, token-redaction, active-server,
  and Media3 playback boundaries.
- Focused unit or screenshot-style validation where practical, plus Android
  compile/test gates.

## Out Of Scope

- V3 irregular geometry, freeform layouts, or bespoke non-Material controls.
- Server administration, metadata editing, provider/addon/webhook/storage UI.
- Cross-device Continue Watching without authoritative User Playback State.
- Downloads/offline playback.
- External player handoff.
- New Public Client API semantics unless split into a server/API workstream.
- A Rust-owned Android player abstraction.
- Copying layouts, assets, code, or branding from Jellyfin, Plex, Findroid, or
  generated references.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| V2 is the initial target, not V3. | High | User preference and ACF handoff. | Re-open design planning before implementation. |
| Current UI may be rewritten without compatibility guarantees. | High | User explicitly prefers clean architecture over preserving old UI. | Keep old route behavior only where it protects API/playback boundaries. |
| Material 3 stable components should be the default foundation. | High | Android foundation and interface design. | Alpha expressive APIs stay optional wrappers. |
| Dynamic color and artwork accents are compatible when scoped separately. | Medium | Interface design says global replacement is deferred, local media accents are allowed. | Add a settings toggle or fallback to static Nako roles. |
| Existing public API is enough for first UI rewrite. | Medium | ACF implemented browse/search/detail/playback decisions. | Split API gaps instead of inventing local pseudo features. |

## Architecture Direction

Keep networking and playback clients intact while replacing UI composition.
The UI rewrite should move toward these package boundaries:

- `ui/theme`: Material theme, static roles, dynamic color opt-in, motion tokens,
  shape/spacing/type tokens.
- `ui/components`: reusable Nako media-client components with no server client
  ownership.
- `ui/shell`: adaptive app chrome, top-level navigation, route transitions.
- `ui/screens`: Home, Libraries, Search, Detail, Facet, Source Picker, Player,
  Settings.
- `ui/state`: screen-facing state holders only when a screen needs more than
  simple local Compose state.

The Public Client API remains the data boundary. UI code may present only
client-safe DTO facts. If a design needs data the API does not expose, show an
explicit API-gap state or split a follow-on API workstream.

## Closeout Condition

This lane can close when:

- V2 Home, Detail, Source Picker, Facet, Settings, and Player surfaces are
  implemented with the shared design-system vocabulary;
- phone and tablet app chrome follow the same product model;
- dynamic color and artwork accent behavior are documented and gated;
- existing connection, browse, search, detail, playback decision, Media3, and
  session-boundary tests still pass;
- final Android build/test gates are fresh;
- remaining API gaps are split into follow-on workstreams instead of hidden in
  client-only behavior.

## Closeout Result

Completed on 2026-05-18.

The Android UI now has the V2 Material 3 Expressive baseline across Home,
Libraries, Browse Facets, Media Item Detail, Source / Version Picker, Player,
Settings Home, and Server Profile. The shipped implementation keeps the V2
regular Compose-friendly geometry, optional dynamic color, local artwork
accents, adaptive phone/tablet chrome, safe diagnostics, and existing Public
Client API and Media3 ownership boundaries.

Deferred follow-ons are explicit:

- V3 irregular/freeform geometry exploration.
- Authoritative User Playback State and cross-device Continue Watching.
- Richer media source technical facts, track/subtitle selection, chapters, and
  source-level diagnostics through Public Client API support.
- Downloads/offline playback, external player handoff, picture-in-picture, and
  advanced player gestures.
- Compose clipboard API migration after the replacement API is adopted in this
  app's Compose baseline.
