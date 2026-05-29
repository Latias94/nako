# Web Playlist Management UI Mutations

Status: Active
Last updated: 2026-05-29

## Why This Lane Exists

The first web playlist slice is intentionally read-oriented. It can list
current-user playlists and playlist items through Public Client live data with
fixture fallback, but users cannot create, rename, delete, add, remove, or
reorder items from the web UI yet.

The backend/Public Client contract and SDK methods already exist. The correct
next step is to wire those mutations into the web product surface with clear
route ownership, optimistic state, error handling, and browser validation
instead of leaving the v0-era static My List affordances around.

## Relevant Authority

- ADRs:
  - `docs/adr/0021-video-first-media-server-domain-model.md`
  - `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md`
  - `docs/adr/0025-openapi-public-client-sdk-contract.md`
  - `docs/adr/0028-user-playback-state-principal-and-public-contract.md`
- Existing docs:
  - `PRODUCT.md`
  - `DESIGN.md`
- Related workstreams:
  - `docs/workstreams/user-playlists-contract-and-web-slice/`
  - `docs/workstreams/web-deferred-product-reentry-plan/`
  - `docs/workstreams/web-media-live-public-client-parity/`

## Problem

Playlist mutation capability exists below the UI, but the web product has no
durable mutation boundary. Without this lane, the page risks accumulating
ad hoc button handlers, fixture-only claims, stale cache behavior, or Admin API
imports that would undermine the Public Client separation.

## Target State

When this lane closes:

- `web/` exposes playlist create, rename, delete, add, remove, and reorder
  controls through Public Client only.
- TanStack Query mutation hooks own cache invalidation, optimistic updates, and
  stale-version/refetch behavior.
- `/media/my-list` keeps route-owned `playlist` and `view` state.
- Mutation controls have explicit loading, empty, conflict, and error states.
- Fixture fallback remains truthful and cannot overclaim live mutation success.
- Route/data-source/state tests and desktop/mobile browser smoke prove the
  management flows.

## In Scope

- `web/src/api/public/media-data-source.ts` playlist mutation methods.
- `web/lib/use-media.ts` TanStack Query mutation hooks and cache policy.
- `web/src/features/media/my-list-page.tsx` management controls.
- Add-to-playlist entry points from media cards/detail where they can be kept
  Public Client-only and narrowly scoped.
- Route/state tests, data-source tests, TypeScript check, bundle budget, and
  browser smoke.

## Out Of Scope

- Shared/public playlists, invites, collaboration, or social discovery.
- Smart playlists, rules, recommendation-generated lists, and auto-curation.
- Offline sync conflict resolution.
- Admin playlist management.
- Backend contract redesign unless implementation reveals a blocking contract
  defect.
- Music queue, podcast queue, or HLS transport playlist behavior.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Public Client mutation routes are already available. | High | `user-playlists-contract-and-web-slice` closed with server/SDK evidence. | Reopen a narrow API/SDK task before UI mutation work. |
| Web can keep this on the existing `createPublicMediaDataSource` boundary. | High | Current read slice already maps playlist DTOs there. | Split a smaller data-source boundary task before UI work. |
| Fixture mode should not pretend mutations are persisted. | High | Product docs require truthful live/mock/fallback states. | Mutation UI must disable or clearly mark fixture-only behavior. |
| Reorder can start as an explicit list action before richer drag-and-drop. | Medium | Route contract accepts full ordered `item_ids`. | DnD can remain a follow-on if keyboard/accessibility cost expands. |

## Architecture Direction

`web/src/api/public` owns SDK calls and DTO-to-UI mapping. `web/lib/use-media.ts`
owns TanStack Query keys and mutation hooks. Feature components call hooks and
render product states, but do not construct Admin API calls or raw fetch
requests.

Cache ownership should stay explicit:

- playlist list key: `["nako", "media", "user-playlists"]`;
- playlist item key: `["nako", "media", "user-playlists", playlistId, "items"]`;
- mutations update or invalidate those keys after success;
- conflict or stale-version responses should refetch before presenting retry.

The UI should keep management controls local to `/media/my-list` first. Add to
playlist from browse/detail can follow once the mutation boundary is stable.

## Refactor Brief

- **Intent**: remove the remaining read-only/static My List limitation and make
  playlist management a Public Client-owned product flow.
- **Scope**: `web/src/api/public`, `web/lib/use-media.ts`,
  `web/src/features/media`, `web/src/shell`, and `web/src/test`.
- **Deletion plan**: remove leftover disabled/static playlist actions when live
  replacements land; do not keep v0-only dropdown affordances that do nothing.
- **Boundary plan**: keep all playlist mutations behind Public Client data
  source methods and TanStack mutation hooks; components own presentation only.
- **Testing plan**: data-source mutation tests, route/state tests, optimistic
  cache tests where practical, full web test/check/build gates, and
  desktop/mobile browser smoke.
- **Risk plan**: preserve truthful fixture fallback, avoid Admin API imports,
  protect inaccessible item facts, and keep conflicts visible instead of silent.
- **Workflow plan**: this is a durable workstream; execute one bounded TODO task
  at a time with `run-workstream-task`.

## Closeout Condition

This lane can close when:

- the target state is implemented;
- evidence gates pass;
- docs reflect shipped behavior;
- mutation UX follow-ons are either split or explicitly deferred.
