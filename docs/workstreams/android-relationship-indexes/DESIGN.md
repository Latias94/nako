# Android Relationship Indexes

Status: Active
Last updated: 2026-05-20

## Why This Lane Exists

Android now has a server-backed Person Detail path from item detail Cast &
Crew. The remaining relationship gap is broader browsing by People, Tags, and
Genres indexes. Those indexes are a product navigation and information
architecture decision, not more Person Detail contract plumbing, so they need a
separate lane.

## Relevant Authority

- ADRs:
  - `docs/adr/0025-openapi-public-client-sdk-contract.md`
  - `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- Existing docs:
  - `docs/api/HTTP_API.md`
  - `docs/workstreams/android-api-contract-integration/`
  - `docs/workstreams/android-material-expressive-ui/`
- Related workstreams:
  - `docs/workstreams/android-api-contract-integration/`

## Problem

Android can open related Media Items for known genre/tag/person IDs, but it has
no first-class People, Tags, or Genres index pages. Users can only discover
those relationships after opening an item. Implementing all indexes inside the
API contract lane would mix product discovery scope with route contract proof
scope and would likely produce thin list screens before the navigation model is
settled.

## Target State

- Android has an explicit decision for People, Tags, and Genres index
  placement in the browse shell.
- Accepted indexes have typed Public Client API list-route client contracts.
- Accepted indexes have route state, save/restore behavior, and Material
  Expressive screens.
- Index rows open existing related Media Items routes and Person Detail routes
  without local filtering.
- Rejected or deferred indexes have a documented rationale and no placeholder
  UI debt.

## ARI-010 Product Decision

| Index | Decision | Rationale | Route placement |
| --- | --- | --- | --- |
| Genres | Accept as first slice. | Genres are stable editorial browsing labels, already visible on item detail, and the existing genre-items route is smoke-proven through the detail facet path. | Add a Home browse anchor that opens a nested Genres Index route; rows open existing Genre related Media Items. |
| Tags | Accept as second slice after Genres. | Tags use the same list-to-related-items shape, but can be noisier and more library-specific than genres. Implement after the shared index route shape is proven. | Reuse the same relationship index screen family; expose as a secondary Home browse anchor only after Genres is stable. |
| People | Defer top-level People index for the initial slice. | Person Detail is already reachable from Cast & Crew, while a useful People index needs stronger role/search semantics to avoid becoming a flat actor-name list. | Keep Person Detail as the primary People path; revisit People index after Genres/Tags and search/filter decisions. |

The first implementation slice is Genres Index. It should not add a new bottom
navigation destination. The current bottom navigation remains Home, Libraries,
Search, and Settings; relationship indexes open as nested browse routes from
Home so the shell does not become cluttered before the IA is proven.

## In Scope

- `GET /people?limit=&offset=`
- `GET /tags?limit=&offset=`
- `GET /genres?limit=&offset=`
- Android browse client DTOs and methods for accepted list routes.
- Browse route state/actions/navigation for accepted indexes.
- Compose screens and smoke assertions for accepted first slice.
- Updating Android API integration docs after the decision.

## Out Of Scope

- Admin/internal routes.
- Server API shape changes unless Android finds a concrete public contract gap.
- Advanced filtering, sorting, multi-select facets, saved filters, or offline
  caching.
- Replacing the existing search route.
- V3 irregular UI exploration.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Genre and Tag indexes are more immediately useful than a People index for small libraries. | Medium | Item detail already exposes genre/tag chips; People can be reached from Cast & Crew. | Prioritize People index first if user testing shows actor/director browsing is the primary entry point. |
| The public list routes expose enough label data for index rows without extra detail calls. | Medium | `docs/api/HTTP_API.md` lists paginated People, Tags, and Genres routes. | Add a client contract spike before UI if response shape is insufficient. |
| Existing `BrowseFacetRouteContent` can remain the related-items destination for index rows. | High | Genre/tag/person item routes are already productized and smoke-covered. | Split related-items route polish as a separate task if index rows need richer context. |

## Architecture Direction

Keep relationship indexes under the existing browse state machine. The screen
should dispatch `BrowseAction` intents, `BrowseSession` should own route state,
and `ClientBrowseDataSource` should be the only UI-facing layer that calls the
Public Client API. Index rows should carry stable server IDs and open existing
facet or Person Detail routes; they must not filter cached item lists locally.

The first implementation slice is Genres Index. Once the Genre list route,
route state, and screen are clean, Tags should reuse the same index route
shape. People remains a Person Detail workflow until a richer top-level People
experience is justified.

## Closeout Condition

This lane can close when:

- the accepted index family set is implemented or explicitly deferred,
- focused Android unit gates cover client contracts and route state,
- at least one accepted index path has smoke or manual evidence,
- docs reflect the shipped behavior,
- and remaining relationship browsing ambitions are split or deferred.
